use askama::Template as _;

use crate::ir::types::{PrimitiveType, TypeExpr};
use crate::ir::{AbiContract, FfiContract};
use crate::render::dart::NamingConvention;
use crate::render::dart::lower::DartLowerer;
use crate::render::dart::templates::EnhancedEnumTemplate;

pub struct DartEmitter {}

impl DartEmitter {
    pub fn emit(
        ffi: &FfiContract,
        abi: &AbiContract,
        package_name: String,
        module_name: String,
    ) -> String {
        let lowerer = DartLowerer::new(ffi, abi, package_name, module_name);
        let library = lowerer.library();

        let mut output = String::new();

        for en in &library.enums {
            let template = match en.kind {
                super::DartEnumKind::CStyle | super::DartEnumKind::Enhanced => {
                    EnhancedEnumTemplate { dart_enum: &en }
                }
                super::DartEnumKind::SealedClass => todo!(),
            };

            output.push_str(&template.render().unwrap());
        }

        output
    }
}

fn render_type_name(name: &str) -> String {
    NamingConvention::class_name(name)
}

pub fn primitive_dart_type(primitive: PrimitiveType) -> String {
    match primitive {
        PrimitiveType::Bool => "bool".to_string(),
        PrimitiveType::I8
        | PrimitiveType::U8
        | PrimitiveType::I16
        | PrimitiveType::U16
        | PrimitiveType::I32
        | PrimitiveType::U32
        | PrimitiveType::I64
        | PrimitiveType::U64
        | PrimitiveType::ISize
        | PrimitiveType::USize => "int".to_string(),
        PrimitiveType::F32 | PrimitiveType::F64 => "double".to_string(),
    }
}

pub fn type_expr_dart_type(ty: &TypeExpr) -> String {
    match ty {
        TypeExpr::Primitive(p) => primitive_dart_type(*p),
        TypeExpr::String => "String".to_string(),
        TypeExpr::Bytes => "Uint8List".to_string(),
        TypeExpr::Vec(inner) => match inner.as_ref() {
            TypeExpr::Primitive(primitive) => match primitive {
                PrimitiveType::I32 => "Int32List".to_string(),
                PrimitiveType::U32 => "Uint32List".to_string(),
                PrimitiveType::I16 => "Int16List".to_string(),
                PrimitiveType::U16 => "Uint16List".to_string(),
                PrimitiveType::I64 => "Int64List".to_string(),
                PrimitiveType::U64 => "Uint64List".to_string(),
                PrimitiveType::ISize => "Int64List".to_string(),
                PrimitiveType::USize => "Uint64List".to_string(),
                PrimitiveType::F32 => "Float32List".to_string(),
                PrimitiveType::F64 => "Float64List".to_string(),
                PrimitiveType::U8 => "Uint8List".to_string(),
                PrimitiveType::I8 => "Int8List".to_string(),
                PrimitiveType::Bool => "Uint8List".to_string(),
            },
            _ => format!("List<{}>", type_expr_dart_type(inner)),
        },
        TypeExpr::Option(inner) => format!("{}?", type_expr_dart_type(inner)),
        TypeExpr::Result { ok, err } => format!(
            "BoltFFIResult<{}, {}>",
            type_expr_dart_type(ok),
            type_expr_dart_type(err)
        ),
        TypeExpr::Record(id) => render_type_name(id.as_str()),
        TypeExpr::Enum(id) => render_type_name(id.as_str()),
        TypeExpr::Custom(id) => render_type_name(id.as_str()),
        TypeExpr::Builtin(id) => match id.as_str() {
            "Duration" => "Duration".to_string(),
            "SystemTime" => "Datetime".to_string(),
            "Uuid" => "(int, int)".to_string(), // NOTE: not builtin
            "Url" => "Uri".to_string(),
            _ => "String".to_string(),
        },
        TypeExpr::Handle(class_id) => render_type_name(class_id.as_str()),
        TypeExpr::Callback(callback_id) => render_type_name(callback_id.as_str()),
        TypeExpr::Void => "void".to_string(),
    }
}
