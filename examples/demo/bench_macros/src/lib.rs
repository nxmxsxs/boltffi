use proc_macro::TokenStream;
use quote::quote;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, parse_quote, Attribute, Error, Ident, ImplItem, Item, LitStr, Result, Token};

enum CandidateKind {
    Function,
    Record,
    Enum,
    Object,
    Impl,
    CallbackInterface,
}

enum TargetKind {
    Uniffi,
    WasmBindgen,
}

enum BoltffiMode {
    Default,
    SingleThreaded,
}

struct CandidateArgs {
    kind: CandidateKind,
    targets: Vec<TargetKind>,
    constructor_name: Option<Ident>,
    boltffi_mode: BoltffiMode,
}

impl Parse for CandidateArgs {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let kind = parse_kind(&input.call(Ident::parse_any)?)?;
        let mut targets = Vec::new();
        let mut constructor_name = None;
        let mut boltffi_mode = BoltffiMode::Default;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            let entry = input.call(Ident::parse_any)?;

            if entry == "constructor" {
                input.parse::<Token![=]>()?;
                let constructor = input.parse::<LitStr>()?;
                constructor_name = Some(Ident::new(&constructor.value(), constructor.span()));
                continue;
            }

            if entry == "boltffi" {
                input.parse::<Token![=]>()?;
                let boltffi_value = input.parse::<LitStr>()?;
                boltffi_mode = parse_boltffi_mode(&boltffi_value)?;
                continue;
            }

            targets.push(parse_target(&entry)?);
        }

        if targets.is_empty() {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "benchmark_candidate requires at least one target",
            ));
        }

        Ok(Self {
            kind,
            targets,
            constructor_name,
            boltffi_mode,
        })
    }
}

#[proc_macro_attribute]
pub fn benchmark_candidate(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as CandidateArgs);
    let item = parse_macro_input!(item as Item);

    expand_candidate(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn expand_candidate(args: CandidateArgs, item: Item) -> Result<proc_macro2::TokenStream> {
    match (args.kind, item) {
        (CandidateKind::Function, Item::Fn(mut item_function)) => {
            prepend_attributes(
                &mut item_function.attrs,
                candidate_attributes(&args.targets, CandidateKind::Function, &args.boltffi_mode),
            );
            Ok(quote!(#item_function))
        }
        (CandidateKind::Record, Item::Struct(mut item_struct)) => {
            prepend_attributes(
                &mut item_struct.attrs,
                candidate_attributes(&args.targets, CandidateKind::Record, &args.boltffi_mode),
            );
            Ok(quote!(#item_struct))
        }
        (CandidateKind::Enum, Item::Enum(mut item_enum)) => {
            prepend_attributes(
                &mut item_enum.attrs,
                candidate_attributes(&args.targets, CandidateKind::Enum, &args.boltffi_mode),
            );
            Ok(quote!(#item_enum))
        }
        (CandidateKind::Object, Item::Struct(mut item_struct)) => {
            prepend_attributes(
                &mut item_struct.attrs,
                candidate_attributes(&args.targets, CandidateKind::Object, &args.boltffi_mode),
            );
            Ok(quote!(#item_struct))
        }
        (CandidateKind::Impl, Item::Impl(mut item_impl)) => {
            prepend_attributes(
                &mut item_impl.attrs,
                candidate_attributes(&args.targets, CandidateKind::Impl, &args.boltffi_mode),
            );

            if let Some(constructor_name) = args.constructor_name {
                let constructor_attributes = constructor_attributes(&args.targets);
                item_impl
                    .items
                    .iter_mut()
                    .filter_map(|impl_item| match impl_item {
                        ImplItem::Fn(method) if method.sig.ident == constructor_name => Some(method),
                        _ => None,
                    })
                    .for_each(|method| prepend_attributes(&mut method.attrs, constructor_attributes.clone()));
            }

            Ok(quote!(#item_impl))
        }
        (CandidateKind::CallbackInterface, Item::Trait(mut item_trait)) => {
            prepend_attributes(
                &mut item_trait.attrs,
                candidate_attributes(&args.targets, CandidateKind::CallbackInterface, &args.boltffi_mode),
            );
            Ok(quote!(#item_trait))
        }
        (kind, item) => Err(Error::new_spanned(
            item,
            format!("benchmark_candidate kind {} does not match item", kind_name(&kind)),
        )),
    }
}

fn prepend_attributes(attributes: &mut Vec<Attribute>, new_attributes: Vec<Attribute>) {
    attributes.splice(0..0, new_attributes);
}

fn candidate_attributes(targets: &[TargetKind], kind: CandidateKind, boltffi_mode: &BoltffiMode) -> Vec<Attribute> {
    let mut attributes = boltffi_attributes(&kind, boltffi_mode);
    let target_attributes = targets.iter().flat_map(|target| match (target, &kind) {
        (TargetKind::Uniffi, CandidateKind::Function) | (TargetKind::Uniffi, CandidateKind::Impl) => {
            vec![parse_quote!(#[cfg_attr(feature = "uniffi", uniffi::export)])]
        }
        (TargetKind::Uniffi, CandidateKind::Record) => {
            vec![parse_quote!(#[cfg_attr(feature = "uniffi", derive(uniffi::Record))])]
        }
        (TargetKind::Uniffi, CandidateKind::Enum) => {
            vec![parse_quote!(#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))])]
        }
        (TargetKind::Uniffi, CandidateKind::Object) => {
            vec![parse_quote!(#[cfg_attr(feature = "uniffi", derive(uniffi::Object))])]
        }
        (TargetKind::Uniffi, CandidateKind::CallbackInterface) => {
            vec![parse_quote!(#[cfg_attr(feature = "uniffi", uniffi::export(callback_interface))])]
        }
        (TargetKind::WasmBindgen, CandidateKind::Function)
        | (TargetKind::WasmBindgen, CandidateKind::Record)
        | (TargetKind::WasmBindgen, CandidateKind::Enum)
        | (TargetKind::WasmBindgen, CandidateKind::Object)
        | (TargetKind::WasmBindgen, CandidateKind::Impl) => {
            vec![parse_quote!(#[cfg_attr(feature = "wasm-bench", wasm_bindgen::prelude::wasm_bindgen)])]
        }
        (TargetKind::WasmBindgen, CandidateKind::CallbackInterface) => Vec::new(),
    });
    attributes.extend(target_attributes);
    attributes
}

fn boltffi_attributes(kind: &CandidateKind, boltffi_mode: &BoltffiMode) -> Vec<Attribute> {
    match kind {
        CandidateKind::Function | CandidateKind::Impl => match boltffi_mode {
            BoltffiMode::Default => {
                vec![parse_quote!(#[cfg_attr(all(not(feature = "uniffi"), not(feature = "wasm-bench")), export)])]
            }
            BoltffiMode::SingleThreaded => {
                vec![
                    parse_quote!(#[cfg_attr(all(not(feature = "uniffi"), not(feature = "wasm-bench")), export(single_threaded))]),
                ]
            }
        },
        CandidateKind::CallbackInterface => {
            vec![parse_quote!(#[cfg_attr(all(not(feature = "uniffi"), not(feature = "wasm-bench")), export)])]
        }
        CandidateKind::Record | CandidateKind::Enum | CandidateKind::Object => Vec::new(),
    }
}

fn constructor_attributes(targets: &[TargetKind]) -> Vec<Attribute> {
    targets
        .iter()
        .flat_map(|target| match target {
            TargetKind::Uniffi => vec![parse_quote!(#[cfg_attr(feature = "uniffi", uniffi::constructor)])],
            TargetKind::WasmBindgen => {
                vec![parse_quote!(#[cfg_attr(feature = "wasm-bench", wasm_bindgen::prelude::wasm_bindgen(constructor))])]
            }
        })
        .collect()
}

fn parse_kind(identifier: &Ident) -> Result<CandidateKind> {
    match identifier.to_string().as_str() {
        "function" => Ok(CandidateKind::Function),
        "record" => Ok(CandidateKind::Record),
        "enum" => Ok(CandidateKind::Enum),
        "object" => Ok(CandidateKind::Object),
        "impl" => Ok(CandidateKind::Impl),
        "callback_interface" => Ok(CandidateKind::CallbackInterface),
        _ => Err(Error::new_spanned(
            identifier,
            "benchmark_candidate kind must be one of function, record, enum, object, impl, callback_interface",
        )),
    }
}

fn parse_target(identifier: &Ident) -> Result<TargetKind> {
    match identifier.to_string().as_str() {
        "uniffi" => Ok(TargetKind::Uniffi),
        "wasm_bindgen" => Ok(TargetKind::WasmBindgen),
        _ => Err(Error::new_spanned(
            identifier,
            "benchmark_candidate target must be one of uniffi or wasm_bindgen",
        )),
    }
}

fn parse_boltffi_mode(mode: &LitStr) -> Result<BoltffiMode> {
    match mode.value().as_str() {
        "default" => Ok(BoltffiMode::Default),
        "single_threaded" => Ok(BoltffiMode::SingleThreaded),
        _ => Err(Error::new_spanned(
            mode,
            "benchmark_candidate boltffi mode must be one of default or single_threaded",
        )),
    }
}

fn kind_name(kind: &CandidateKind) -> &'static str {
    match kind {
        CandidateKind::Function => "function",
        CandidateKind::Record => "record",
        CandidateKind::Enum => "enum",
        CandidateKind::Object => "object",
        CandidateKind::Impl => "impl",
        CandidateKind::CallbackInterface => "callback_interface",
    }
}
