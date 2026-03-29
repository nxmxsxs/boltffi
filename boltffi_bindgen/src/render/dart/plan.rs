use crate::{
    Primitive,
    ir::{AbiCall, AbiParam, AbiType, ErrorTransport, ParamRole, Transport},
};

#[derive(Debug, Clone)]
pub struct DartLibrary {
    pub native: DartNative,
    pub enums: Vec<DartEnum>,
    pub records: Vec<DartRecord>,
}

#[derive(Debug, Clone, Copy)]
pub enum DartEnumKind {
    CStyle,
    Enhanced,
    SealedClass,
}

#[derive(Debug, Clone)]
pub struct DartEnumField {
    pub name: String,
    pub dart_type: String,
    pub wire_decode_expr: String,
    pub wire_size_expr: String,
    pub wire_encode_expr: String,
}

#[derive(Debug, Clone)]
pub struct DartEnumVariant {
    pub name: String,
    pub class_name: String,
    pub tag: i128,
    pub fields: Vec<DartEnumField>,
}

#[derive(Debug, Clone)]
pub struct DartEnum {
    pub name: String,
    pub kind: DartEnumKind,
    pub tag_type: String,
    pub variants: Vec<DartEnumVariant>,
}

#[derive(Debug, Clone)]
pub struct DartRecordField {
    pub name: String,
    pub offset: usize,
    pub dart_type: String,
    pub wire_decode_expr: String,
    pub wire_encode_expr: String,
}


#[derive(Debug, Clone)]
pub struct DartBlittableLayout {
    pub struct_size: usize,
    pub fields: Vec<DartBlittableField>,
}

#[derive(Debug, Clone)]
pub struct DartBlittableField {
    pub name: String,
    pub native_type: DartNativeType,
    pub const_name: String,
    pub offset: usize,
    pub decode_expr: String,
    pub encode_expr: String,
}

#[derive(Debug, Clone)]
pub struct DartRecord {
    pub name: String,
    pub fields: Vec<DartRecordField>,
    pub blittable_layout: Option<DartBlittableLayout>,
}

#[derive(Debug, Clone)]
pub struct DartNative {
    pub functions: Vec<DartNativeFunction>,
}

#[derive(Clone, Debug)]
pub enum DartNativeType {
    Void,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    IntPtr,
    UintPtr,
    Float,
    Double,
    Function {
        params: Vec<DartNativeType>,
        return_ty: Box<DartNativeType>,
    },
    Pointer(Box<DartNativeType>),
    Custom(String),
}

impl DartNativeType {
    pub fn from_primitive(primitive: &Primitive) -> Self {
        match primitive {
            Primitive::Bool => DartNativeType::Bool,
            Primitive::I8 => DartNativeType::Int8,
            Primitive::U8 => DartNativeType::Uint8,
            Primitive::I16 => DartNativeType::Int16,
            Primitive::U16 => DartNativeType::Uint16,
            Primitive::I32 => DartNativeType::Int32,
            Primitive::U32 => DartNativeType::Uint32,
            Primitive::I64 => DartNativeType::Int64,
            Primitive::U64 => DartNativeType::Uint64,
            Primitive::ISize => DartNativeType::IntPtr,
            Primitive::USize => DartNativeType::UintPtr,
            Primitive::F32 => DartNativeType::Float,
            Primitive::F64 => DartNativeType::Double,
        }
    }
    pub fn from_abi_type(abi_type: &AbiType) -> Self {
        match abi_type {
            AbiType::Void => DartNativeType::Void,
            AbiType::Bool => DartNativeType::Bool,
            AbiType::I8 => DartNativeType::Int8,
            AbiType::U8 => DartNativeType::Uint8,
            AbiType::I16 => DartNativeType::Int16,
            AbiType::U16 => DartNativeType::Uint16,
            AbiType::I32 => DartNativeType::Int32,
            AbiType::U32 => DartNativeType::Uint32,
            AbiType::I64 => DartNativeType::Int64,
            AbiType::U64 => DartNativeType::Uint64,
            AbiType::ISize => DartNativeType::IntPtr,
            AbiType::USize => DartNativeType::UintPtr,
            AbiType::F32 => DartNativeType::Float,
            AbiType::F64 => DartNativeType::Double,
            AbiType::Pointer(primitive) => {
                DartNativeType::Pointer(Box::new(Self::from_primitive(primitive)))
            }
            AbiType::OwnedBuffer => DartNativeType::Custom("FfiBuf_u8".to_string()),
            AbiType::InlineCallbackFn {
                params,
                return_type,
            } => DartNativeType::Function {
                params: params.iter().map(|p| Self::from_abi_type(p)).collect(),
                return_ty: Box::new(Self::from_abi_type(return_type)),
            },
            AbiType::Handle(class_id) => {
                DartNativeType::Custom("ffi.Pointer<ffi.Void>".to_string())
            }
            AbiType::CallbackHandle => DartNativeType::Custom("BoltFFICallbackHandle".to_string()),
            AbiType::Struct(record_id) => {
                DartNativeType::Custom(format!("___{}", record_id.as_str()))
            }
        }
    }
    pub fn native_type(&self) -> String {
        match self {
            DartNativeType::Void => "ffi.Void".to_string(),
            DartNativeType::Bool => "ffi.Bool".to_string(),
            DartNativeType::Int8 => "ffi.Int8".to_string(),
            DartNativeType::Int16 => "ffi.Int16".to_string(),
            DartNativeType::Int32 => "ffi.Int32".to_string(),
            DartNativeType::Int64 => "ffi.Int64".to_string(),
            DartNativeType::Uint8 => "ffi.Uint8".to_string(),
            DartNativeType::Uint16 => "ffi.Uint16".to_string(),
            DartNativeType::Uint32 => "ffi.Uint32".to_string(),
            DartNativeType::Uint64 => "ffi.Uint64".to_string(),
            DartNativeType::IntPtr => "ffi.IntPtr".to_string(),
            DartNativeType::UintPtr => "ffi.UintPtr".to_string(),
            DartNativeType::Float => "ffi.Float".to_string(),
            DartNativeType::Double => "ffi.Double".to_string(),
            DartNativeType::Function { params, return_ty } => format!(
                "ffi.Pointer<ffi.NativeFunction<{} Function({})>>",
                return_ty.native_type(),
                std::iter::chain(
                    std::iter::once("ffi.Pointer<ffi.Void>".to_string()),
                    params.iter().map(|p| p.native_type())
                )
                .fold(String::new(), |acc, p| if acc.is_empty() {
                    p
                } else {
                    acc + ", " + p.as_str()
                })
            ),
            DartNativeType::Pointer(inner) => format!("ffi.Pointer<{}>", inner.native_type()),
            DartNativeType::Custom(ty) => ty.clone(),
        }
    }

    pub fn dart_sub_type(&self) -> String {
        match self {
            DartNativeType::Void => "void".to_string(),
            DartNativeType::Bool => "bool".to_string(),
            DartNativeType::Int8
            | DartNativeType::Int16
            | DartNativeType::Int32
            | DartNativeType::Int64
            | DartNativeType::Uint8
            | DartNativeType::Uint16
            | DartNativeType::Uint32
            | DartNativeType::Uint64
            | DartNativeType::IntPtr
            | DartNativeType::UintPtr => "int".to_string(),
            DartNativeType::Float | DartNativeType::Double => "double".to_string(),
            o @ DartNativeType::Function { .. } => o.native_type(),
            DartNativeType::Pointer(inner) => format!("ffi.Pointer<{}>", inner.native_type()),
            DartNativeType::Custom(ty) => ty.clone(),
        }
    }

    pub fn abi_call_return_type(abi_call: &AbiCall) -> Self {
        if let Some(Transport::Handle { class_id, .. }) = &abi_call.returns.transport {
            return Self::from_abi_type(&AbiType::Handle(class_id.clone()));
        }

        if matches!(abi_call.returns.transport, Some(Transport::Callback { .. })) {
            return Self::from_abi_type(&AbiType::CallbackHandle);
        }

        if matches!(abi_call.error, ErrorTransport::Encoded { .. }) {
            return Self::from_abi_type(&AbiType::OwnedBuffer);
        }

        match &abi_call.returns.transport {
            None => {
                if matches!(abi_call.error, ErrorTransport::StatusCode) {
                    Self::Custom("FfiStatus".to_string())
                } else {
                    Self::from_abi_type(&AbiType::Void)
                }
            }
            Some(Transport::Scalar(origin)) => Self::from_primitive(&origin.primitive()),
            Some(Transport::Composite(layout)) => {
                Self::from_abi_type(&AbiType::Struct(layout.record_id.clone()))
            }
            Some(Transport::Span(_)) => Self::from_abi_type(&AbiType::OwnedBuffer),
            Some(Transport::Handle { .. } | Transport::Callback { .. }) => unreachable!(),
        }
    }

    pub fn from_abi_param(abi_param: &AbiParam) -> Self {
        let native_type = Self::from_abi_type(&abi_param.abi_type);

        match &abi_param.role {
            ParamRole::OutDirect | ParamRole::OutLen { .. } => Self::Pointer(Box::new(native_type)),
            ParamRole::CallbackContext { .. } => Self::Custom("ffi.Pointer<ffi.Void>".to_string()),
            _ => native_type,
        }
    }

    pub fn field_annot(&self) -> String {
        match self {
            DartNativeType::Void => String::new(),
            DartNativeType::Bool => "@ffi.Bool()".to_string(),
            DartNativeType::Int8 => "@ffi.Int8()".to_string(),
            DartNativeType::Int16 => "@ffi.Int16()".to_string(),
            DartNativeType::Int32 => "@ffi.Int32()".to_string(),
            DartNativeType::Int64 => "@ffi.Int64()".to_string(),
            DartNativeType::Uint8 => "@ffi.Uint8()".to_string(),
            DartNativeType::Uint16 => "@ffi.Uint16()".to_string(),
            DartNativeType::Uint32 => "@ffi.Uint32()".to_string(),
            DartNativeType::Uint64 => "@ffi.Uint64()".to_string(),
            DartNativeType::IntPtr => "@ffi.IntPtr()".to_string(),
            DartNativeType::UintPtr => "@ffi.UintPtr()".to_string(),
            DartNativeType::Float => "@ffi.Float()".to_string(),
            DartNativeType::Double => "@ffi.Double()".to_string(),
            DartNativeType::Function { params, return_ty } => String::new(),
            DartNativeType::Pointer(inner) => String::new(),
            DartNativeType::Custom(_) => String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DartNativeFunctionParam {
    pub name: String,
    pub native_type: DartNativeType,
}

#[derive(Debug, Clone)]
pub struct DartNativeFunction {
    pub symbol: String,
    pub params: Vec<DartNativeFunctionParam>,
    pub return_type: DartNativeType,
    pub is_leaf: bool,
}
