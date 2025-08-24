use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, ItemFn, Pat, ReturnType, Type};

#[proc_macro_derive(FfiType)]
pub fn derive_ffi_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let has_repr_c = input.attrs.iter().any(|attr| {
        attr.path().is_ident("repr")
            && attr
                .parse_args::<syn::Ident>()
                .map(|id| id == "C")
                .unwrap_or(false)
    });

    if !has_repr_c {
        return syn::Error::new_spanned(&input, "FfiType requires #[repr(C)]")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {};

    TokenStream::from(expanded)
}

fn extract_arg_idents(inputs: &syn::punctuated::Punctuated<FnArg, syn::Token![,]>) -> Vec<&Pat> {
    inputs
        .iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                Some(pat_type.pat.as_ref())
            } else {
                None
            }
        })
        .collect()
}

fn is_string_return(output: &ReturnType) -> bool {
    match output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => {
            let type_str = quote::quote!(#ty).to_string();
            type_str == "String" || type_str == "std :: string :: String"
        }
    }
}

fn get_return_type(output: &ReturnType) -> Option<&Type> {
    match output {
        ReturnType::Default => None,
        ReturnType::Type(_, ty) => Some(ty.as_ref()),
    }
}

#[proc_macro_attribute]
pub fn ffi_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let fn_inputs = &input.sig.inputs;
    let fn_output = &input.sig.output;
    let fn_vis = &input.vis;

    let arg_idents = extract_arg_idents(fn_inputs);

    let export_name = format!("mffi_{}", fn_name);
    let export_ident = syn::Ident::new(&export_name, fn_name.span());

    let expanded = if is_string_return(fn_output) {
        quote! {
            #input

            #[unsafe(no_mangle)]
            #fn_vis unsafe extern "C" fn #export_ident(
                #fn_inputs,
                out: *mut crate::FfiString
            ) -> crate::FfiStatus {
                if out.is_null() {
                    return crate::FfiStatus::NULL_POINTER;
                }
                let result = #fn_name(#(#arg_idents),*);
                *out = crate::FfiString::from(result);
                crate::FfiStatus::OK
            }
        }
    } else {
        quote! {
            #input

            #[unsafe(no_mangle)]
            #fn_vis extern "C" fn #export_ident(#fn_inputs) #fn_output {
                #fn_name(#(#arg_idents),*)
            }
        }
    };

    TokenStream::from(expanded)
}
