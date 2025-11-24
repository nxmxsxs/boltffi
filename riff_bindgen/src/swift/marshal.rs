use crate::model::{Primitive, Type};
use riff_ffi_rules::naming;

use super::names::NamingConvention;

#[derive(Debug, Clone, PartialEq)]
pub enum SwiftType {
    Void,
    Primitive(Primitive),
    String,
    Bytes,
    Slice { inner: Box<SwiftType>, mutable: bool },
    Vec(Box<SwiftType>),
    Option(Box<SwiftType>),
    Result { ok: Box<SwiftType> },
    Enum(String),
    Record(String),
    Object(String),
    BoxedTrait(String),
    Callback(Box<SwiftType>),
}

impl SwiftType {
    pub fn from_model(ty: &Type) -> Self {
        match ty {
            Type::Void => Self::Void,
            Type::Primitive(p) => Self::Primitive(*p),
            Type::String => Self::String,
            Type::Bytes => Self::Bytes,
            Type::Slice(inner) => Self::Slice {
                inner: Box::new(Self::from_model(inner)),
                mutable: false,
            },
            Type::MutSlice(inner) => Self::Slice {
                inner: Box::new(Self::from_model(inner)),
                mutable: true,
            },
            Type::Vec(inner) => Self::Vec(Box::new(Self::from_model(inner))),
            Type::Option(inner) => Self::Option(Box::new(Self::from_model(inner))),
            Type::Result { ok, .. } => Self::Result {
                ok: Box::new(Self::from_model(ok)),
            },
            Type::Enum(name) => Self::Enum(name.clone()),
            Type::Record(name) => Self::Record(name.clone()),
            Type::Object(name) => Self::Object(name.clone()),
            Type::BoxedTrait(name) => Self::BoxedTrait(name.clone()),
            Type::Callback(inner) => Self::Callback(Box::new(Self::from_model(inner))),
        }
    }

    pub fn swift_type(&self) -> String {
        match self {
            Self::Void => "Void".into(),
            Self::Primitive(p) => p.swift_type().into(),
            Self::String => "String".into(),
            Self::Bytes => "Data".into(),
            Self::Slice { inner, .. } | Self::Vec(inner) => format!("[{}]", inner.swift_type()),
            Self::Option(inner) => format!("{}?", inner.swift_type()),
            Self::Result { ok } => ok.swift_type(),
            Self::Enum(name) | Self::Record(name) | Self::Object(name) => {
                NamingConvention::class_name(name)
            }
            Self::BoxedTrait(name) => format!("{}Protocol", NamingConvention::class_name(name)),
            Self::Callback(inner) => format!("({}) -> Void", inner.swift_type()),
        }
    }

    pub fn default_value(&self) -> String {
        match self {
            Self::Void => "()".into(),
            Self::Primitive(p) => p.default_value().into(),
            Self::String => "\"\"".into(),
            Self::Bytes => "Data()".into(),
            Self::Slice { .. } | Self::Vec(_) => "[]".into(),
            Self::Option(_) => "nil".into(),
            Self::Result { ok } => ok.default_value(),
            Self::Enum(_) => "0".into(),
            Self::Record(name) => format!("{}()", NamingConvention::class_name(name)),
            Self::Object(_) | Self::BoxedTrait(_) => "nil".into(),
            Self::Callback(_) => "{ _ in }".into(),
        }
    }

    pub fn ffi_type_suffix(&self) -> String {
        match self {
            Self::Primitive(p) => p.rust_name().into(),
            Self::String => "string".into(),
            Self::Record(name) | Self::Enum(name) => name.to_lowercase(),
            Self::Vec(inner) => inner.ffi_type_suffix(),
            Self::Result { ok } => ok.ffi_type_suffix(),
            _ => "unknown".into(),
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Record(_))
    }

    pub fn unwrap_result(&self) -> &SwiftType {
        match self {
            Self::Result { ok } => ok.as_ref(),
            other => other,
        }
    }

    pub fn inner_type(&self) -> Option<&SwiftType> {
        match self {
            Self::Vec(inner) | Self::Option(inner) | Self::Result { ok: inner } => Some(inner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReturnStrategy {
    Direct,
    DirectEnum { type_name: String },
    DirectRecord { type_name: String },
    String,
    Vec { inner_type: String, inner_is_struct: bool },
    Option { inner_type: String },
    Result { inner: Box<ReturnStrategy> },
    Void,
    VoidChecked,
}

impl ReturnStrategy {
    pub fn from_type(ty: &SwiftType) -> Self {
        match ty {
            SwiftType::Void => Self::Void,
            SwiftType::Primitive(_) | SwiftType::Object(_) | SwiftType::BoxedTrait(_) => {
                Self::Direct
            }
            SwiftType::String => Self::String,
            SwiftType::Enum(name) => Self::DirectEnum {
                type_name: NamingConvention::class_name(name),
            },
            SwiftType::Record(name) => Self::DirectRecord {
                type_name: NamingConvention::class_name(name),
            },
            SwiftType::Vec(inner) => Self::Vec {
                inner_type: inner.swift_type(),
                inner_is_struct: inner.is_struct(),
            },
            SwiftType::Option(inner) => Self::Option {
                inner_type: inner.swift_type(),
            },
            SwiftType::Result { ok } => Self::Result {
                inner: Box::new(Self::from_type(ok)),
            },
            _ => Self::Direct,
        }
    }

    pub fn throws(&self) -> bool {
        matches!(self, Self::Result { .. })
    }

    pub fn has_out_param(&self) -> bool {
        matches!(
            self,
            Self::String | Self::Option { .. } | Self::Result { .. }
        )
    }

    pub fn needs_status_check(&self) -> bool {
        matches!(
            self,
            Self::String | Self::Vec { .. } | Self::VoidChecked | Self::Result { .. }
        )
    }

    pub fn inner_strategy(&self) -> Option<&ReturnStrategy> {
        match self {
            Self::Result { inner } => Some(inner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamConversion {
    pub name: String,
    pub swift_type: String,
    pub wrapper_pre: Option<String>,
    pub wrapper_post: Option<String>,
    pub ffi_args: Vec<String>,
    pub is_mutable: bool,
    pub is_escaping: bool,
}

impl ParamConversion {
    pub fn from_param(name: &str, ty: &Type) -> Self {
        let swift_ty = SwiftType::from_model(ty);
        let swift_name = NamingConvention::param_name(name);
        
        let (wrapper_pre, ffi_args, wrapper_post, is_mutable) = match &swift_ty {
            SwiftType::String => (
                Some(format!("{}.withCString {{ {}Ptr in", swift_name, swift_name)),
                vec![
                    format!("UnsafeRawPointer({}Ptr).assumingMemoryBound(to: UInt8.self)", swift_name),
                    format!("UInt({}.utf8.count)", swift_name),
                ],
                Some("}".into()),
                false,
            ),
            SwiftType::Bytes => (
                Some(format!("{}.withUnsafeBytes {{ {}Ptr in", swift_name, swift_name)),
                vec![
                    format!("{}Ptr.baseAddress!.assumingMemoryBound(to: UInt8.self)", swift_name),
                    format!("UInt({}.count)", swift_name),
                ],
                Some("}".into()),
                false,
            ),
            SwiftType::Slice { mutable: false, .. } | SwiftType::Vec(_) => (
                Some(format!("{}.withUnsafeBufferPointer {{ {}Ptr in", swift_name, swift_name)),
                vec![
                    format!("{}Ptr.baseAddress", swift_name),
                    format!("UInt({}Ptr.count)", swift_name),
                ],
                Some("}".into()),
                false,
            ),
            SwiftType::Slice { mutable: true, .. } => (
                Some(format!("{}.withUnsafeMutableBufferPointer {{ {}Ptr in", swift_name, swift_name)),
                vec![
                    format!("{}Ptr.baseAddress", swift_name),
                    format!("UInt({}Ptr.count)", swift_name),
                ],
                Some("}".into()),
                true,
            ),
            SwiftType::Enum(_) => (
                None,
                vec![format!("{}.cValue", swift_name)],
                None,
                false,
            ),
            SwiftType::BoxedTrait(trait_name) => (
                None,
                vec![format!("{}Bridge.create({})", NamingConvention::class_name(trait_name), swift_name)],
                None,
                false,
            ),
            _ => (None, vec![swift_name.clone()], None, false),
        };

        Self {
            name: swift_name,
            swift_type: swift_ty.swift_type(),
            wrapper_pre,
            wrapper_post,
            ffi_args,
            is_mutable,
            is_escaping: matches!(swift_ty, SwiftType::Callback(_)),
        }
    }

    pub fn needs_wrapper(&self) -> bool {
        self.wrapper_pre.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct ReturnConversion {
    pub swift_type: SwiftType,
    pub strategy: ReturnStrategy,
    pub display_type: String,
}

impl ReturnConversion {
    pub fn from_type(ty: Option<&Type>) -> Self {
        let swift_type = ty.map(SwiftType::from_model).unwrap_or(SwiftType::Void);
        let strategy = ReturnStrategy::from_type(&swift_type);
        let display_type = match &swift_type {
            SwiftType::Result { ok } => ok.swift_type(),
            other => other.swift_type(),
        };

        Self {
            swift_type,
            strategy,
            display_type,
        }
    }

    pub fn is_void(&self) -> bool {
        self.swift_type.is_void()
    }

    pub fn throws(&self) -> bool {
        self.strategy.throws()
    }

    pub fn free_fn(&self) -> Option<String> {
        let prefix = naming::ffi_prefix();
        match &self.swift_type {
            SwiftType::String => Some(format!("{}_free_string", prefix)),
            SwiftType::Vec(inner) => {
                Some(format!("{}_free_buf_{}", prefix, inner.ffi_type_suffix()))
            }
            SwiftType::Result { ok } => {
                let inner_ret = ReturnConversion {
                    swift_type: ok.as_ref().clone(),
                    strategy: ReturnStrategy::from_type(ok),
                    display_type: ok.swift_type(),
                };
                inner_ret.free_fn()
            }
            _ => None,
        }
    }

    pub fn inner_type(&self) -> Option<String> {
        self.swift_type.inner_type().map(|t| t.swift_type())
    }

    pub fn inner_is_struct(&self) -> bool {
        self.swift_type
            .inner_type()
            .map(|t| t.is_struct())
            .unwrap_or(false)
    }

    pub fn default_value(&self) -> String {
        self.swift_type.unwrap_result().default_value()
    }

    pub fn convert_ffi_result(&self, expr: &str) -> String {
        match &self.strategy {
            ReturnStrategy::Direct => expr.into(),
            ReturnStrategy::DirectEnum { type_name } => {
                format!("{}(fromC: {})", type_name, expr)
            }
            ReturnStrategy::DirectRecord { type_name } => {
                format!("{}(fromC: {})", type_name, expr)
            }
            ReturnStrategy::String => format!("stringFromFfi({})", expr),
            ReturnStrategy::Vec { .. } => {
                format!("Array(UnsafeBufferPointer(start: {}.ptr, count: Int({}.len)))", expr, expr)
            }
            ReturnStrategy::Option { inner_type } => {
                format!("{} != 0 ? {} as {} : nil", expr, expr, inner_type)
            }
            ReturnStrategy::Result { inner } => {
                let inner_conv = ReturnConversion {
                    swift_type: self.swift_type.unwrap_result().clone(),
                    strategy: inner.as_ref().clone(),
                    display_type: self.display_type.clone(),
                };
                inner_conv.convert_ffi_result(expr)
            }
            ReturnStrategy::Void | ReturnStrategy::VoidChecked => String::new(),
        }
    }

    pub fn generate_sync_return(&self, ffi_call: &str) -> SyncReturnCode {
        match &self.strategy {
            ReturnStrategy::Direct => SyncReturnCode {
                declarations: vec![],
                call_expr: format!("return {}", ffi_call),
                post_call: vec![],
            },
            ReturnStrategy::DirectEnum { type_name } => SyncReturnCode {
                declarations: vec![],
                call_expr: format!("return {}(fromC: {})", type_name, ffi_call),
                post_call: vec![],
            },
            ReturnStrategy::DirectRecord { type_name } => SyncReturnCode {
                declarations: vec![],
                call_expr: format!("return {}(fromC: {})", type_name, ffi_call),
                post_call: vec![],
            },
            ReturnStrategy::String => {
                let free_fn = self.free_fn().unwrap_or_default();
                SyncReturnCode {
                    declarations: vec![
                        "var result = FfiString(ptr: nil, len: 0, cap: 0)".into(),
                    ],
                    call_expr: format!("let status = {}, &result)", ffi_call.trim_end_matches(')')),
                    post_call: vec![
                        format!("defer {{ {}(result) }}", free_fn),
                        "ensureOk(status)".into(),
                        "return stringFromFfi(result)".into(),
                    ],
                }
            }
            ReturnStrategy::Option { inner_type } => SyncReturnCode {
                declarations: vec![
                    format!("var outValue: {} = {}", inner_type, self.default_value()),
                ],
                call_expr: format!("let isSome = {}, &outValue)", ffi_call.trim_end_matches(')')),
                post_call: vec![
                    "return isSome != 0 ? outValue : nil".into(),
                ],
            },
            ReturnStrategy::Result { .. } => SyncReturnCode {
                declarations: vec![
                    format!("var outValue: {} = {}", self.display_type, self.default_value()),
                ],
                call_expr: format!("let status = {}, &outValue)", ffi_call.trim_end_matches(')')),
                post_call: vec![
                    "try checkStatus(status)".into(),
                    "return outValue".into(),
                ],
            },
            ReturnStrategy::Void => SyncReturnCode {
                declarations: vec![],
                call_expr: ffi_call.into(),
                post_call: vec![],
            },
            ReturnStrategy::VoidChecked => SyncReturnCode {
                declarations: vec![],
                call_expr: format!("let status = {}", ffi_call),
                post_call: vec!["ensureOk(status)".into()],
            },
            ReturnStrategy::Vec { inner_type, inner_is_struct } => {
                if *inner_is_struct {
                    SyncReturnCode {
                        declarations: vec![],
                        call_expr: "// Vec of structs requires two-phase copy".into(),
                        post_call: vec![],
                    }
                } else {
                    SyncReturnCode {
                        declarations: vec![
                            format!("var arr = [{}](repeating: {}, count: Int(len))", inner_type, self.default_value()),
                            "var written: UInt = 0".into(),
                        ],
                        call_expr: "// Vec requires ffi_vec_len + ffi_vec_copy_into".into(),
                        post_call: vec![
                            "ensureOk(status)".into(),
                            "let writtenCount = min(Int(written), arr.count)".into(),
                            "if writtenCount < arr.count { arr.removeSubrange(writtenCount..<arr.count) }".into(),
                            "return arr".into(),
                        ],
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncReturnCode {
    pub declarations: Vec<String>,
    pub call_expr: String,
    pub post_call: Vec<String>,
}

impl SyncReturnCode {
    pub fn declarations_str(&self) -> String {
        self.declarations.join("\n")
    }

    pub fn post_call_str(&self) -> String {
        self.post_call.join("\n")
    }
}

pub struct SyncCallBuilder {
    ffi_name: String,
    params: Vec<ParamConversion>,
    ret: ReturnConversion,
    include_handle: bool,
}

impl SyncCallBuilder {
    pub fn new(ffi_name: &str, include_handle: bool) -> Self {
        Self {
            ffi_name: ffi_name.into(),
            params: Vec::new(),
            ret: ReturnConversion::from_type(None),
            include_handle,
        }
    }

    pub fn with_params<'a>(mut self, params: impl Iterator<Item = (&'a str, &'a Type)>) -> Self {
        self.params = params.map(|(n, t)| ParamConversion::from_param(n, t)).collect();
        self
    }

    pub fn with_return(mut self, ty: Option<&Type>) -> Self {
        self.ret = ReturnConversion::from_type(ty);
        self
    }

    pub fn build_wrappers_open(&self) -> String {
        self.params
            .iter()
            .filter_map(|p| p.wrapper_pre.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn build_wrappers_close(&self) -> String {
        self.params
            .iter()
            .filter_map(|p| p.wrapper_post.as_ref())
            .rev()
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn build_ffi_args(&self) -> String {
        let mut args = Vec::new();
        if self.include_handle {
            args.push("handle".into());
        }
        for param in &self.params {
            args.extend(param.ffi_args.clone());
        }
        args.join(", ")
    }

    pub fn build_call(&self) -> String {
        format!("{}({})", self.ffi_name, self.build_ffi_args())
    }

    pub fn has_wrappers(&self) -> bool {
        self.params.iter().any(|p| p.needs_wrapper())
    }

    pub fn return_conversion(&self) -> &ReturnConversion {
        &self.ret
    }

    pub fn params(&self) -> &[ParamConversion] {
        &self.params
    }
}
