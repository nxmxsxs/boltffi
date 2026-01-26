use askama::Template;

use super::plan::{
    SwiftCallMode, SwiftCallback, SwiftClass, SwiftEnum, SwiftField, SwiftFunction, SwiftRecord,
    SwiftStreamMode, SwiftVariant,
};

#[derive(Template)]
#[template(path = "preamble.txt", escape = "none")]
pub struct PreambleTemplate<'a> {
    pub prefix: &'a str,
    pub ffi_module_name: Option<&'a str>,
    pub has_async: bool,
    pub has_streams: bool,
}

impl<'a> PreambleTemplate<'a> {
    pub fn new(
        prefix: &'a str,
        ffi_module_name: Option<&'a str>,
        has_async: bool,
        has_streams: bool,
    ) -> Self {
        Self {
            prefix,
            ffi_module_name,
            has_async,
            has_streams,
        }
    }
}

pub fn render_preamble(
    prefix: &str,
    ffi_module_name: Option<&str>,
    has_async: bool,
    has_streams: bool,
) -> String {
    PreambleTemplate::new(prefix, ffi_module_name, has_async, has_streams)
        .render()
        .unwrap()
}

#[derive(Template)]
#[template(path = "record.txt", escape = "none")]
pub struct RecordTemplate<'a> {
    pub class_name: &'a str,
    pub fields: &'a [SwiftField],
    pub is_blittable: bool,
    pub blittable_size: Option<usize>,
}

impl<'a> RecordTemplate<'a> {
    pub fn from_record(record: &'a SwiftRecord) -> Self {
        Self {
            class_name: &record.class_name,
            fields: &record.fields,
            is_blittable: record.is_blittable,
            blittable_size: record.blittable_size,
        }
    }
}

#[derive(Template)]
#[template(path = "enum_c_style.txt", escape = "none")]
pub struct EnumCStyleTemplate<'a> {
    pub class_name: &'a str,
    pub variants: &'a [SwiftVariant],
    pub is_error: bool,
}

impl<'a> EnumCStyleTemplate<'a> {
    pub fn from_enum(e: &'a SwiftEnum) -> Self {
        Self {
            class_name: &e.name,
            variants: &e.variants,
            is_error: e.is_error,
        }
    }
}

#[derive(Template)]
#[template(path = "enum_data.txt", escape = "none")]
pub struct EnumDataTemplate<'a> {
    pub class_name: &'a str,
    pub variants: &'a [SwiftVariant],
    pub is_error: bool,
}

impl<'a> EnumDataTemplate<'a> {
    pub fn from_enum(e: &'a SwiftEnum) -> Self {
        Self {
            class_name: &e.name,
            variants: &e.variants,
            is_error: e.is_error,
        }
    }
}

pub fn render_record(record: &SwiftRecord) -> String {
    RecordTemplate::from_record(record).render().unwrap()
}

pub fn render_enum(e: &SwiftEnum) -> String {
    if e.is_c_style {
        EnumCStyleTemplate::from_enum(e).render().unwrap()
    } else {
        EnumDataTemplate::from_enum(e).render().unwrap()
    }
}

#[derive(Template)]
#[template(path = "callback_trait.txt", escape = "none")]
pub struct CallbackTemplate<'a> {
    pub callback: &'a SwiftCallback,
}

impl<'a> CallbackTemplate<'a> {
    pub fn new(callback: &'a SwiftCallback) -> Self {
        Self { callback }
    }
}

pub fn render_callback(callback: &SwiftCallback) -> String {
    CallbackTemplate::new(callback).render().unwrap()
}

#[derive(Template)]
#[template(path = "function.txt", escape = "none")]
pub struct FunctionTemplate<'a> {
    pub func: &'a SwiftFunction,
    pub prefix: &'a str,
}

impl<'a> FunctionTemplate<'a> {
    pub fn new(func: &'a SwiftFunction, prefix: &'a str) -> Self {
        Self { func, prefix }
    }
}

pub fn render_function(func: &SwiftFunction, prefix: &str) -> String {
    FunctionTemplate::new(func, prefix).render().unwrap()
}

#[derive(Template)]
#[template(path = "class.txt", escape = "none")]
pub struct ClassTemplate<'a> {
    pub cls: &'a SwiftClass,
    pub prefix: &'a str,
}

impl<'a> ClassTemplate<'a> {
    pub fn new(cls: &'a SwiftClass, prefix: &'a str) -> Self {
        Self { cls, prefix }
    }
}

pub fn render_class(cls: &SwiftClass, prefix: &str) -> String {
    ClassTemplate::new(cls, prefix).render().unwrap()
}

use super::plan::SwiftModule;

pub struct SwiftEmitter {
    prefix: String,
    ffi_module_name: Option<String>,
}

impl SwiftEmitter {
    pub fn new() -> Self {
        Self {
            prefix: String::new(),
            ffi_module_name: None,
        }
    }

    pub fn with_prefix(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            ffi_module_name: None,
        }
    }

    pub fn with_ffi_module(mut self, ffi_module: impl Into<String>) -> Self {
        self.ffi_module_name = Some(ffi_module.into());
        self
    }

    pub fn emit(&self, module: &SwiftModule) -> String {
        let mut output = String::new();

        output.push_str(&render_preamble(
            &self.prefix,
            self.ffi_module_name.as_deref(),
            module.has_async(),
            module.has_streams(),
        ));
        output.push_str("\n\n");

        for record in &module.records {
            output.push_str(&render_record(record));
            output.push_str("\n\n");
        }

        for e in &module.enums {
            output.push_str(&render_enum(e));
            output.push_str("\n\n");
        }

        for callback in &module.callbacks {
            output.push_str(&render_callback(callback));
            output.push_str("\n\n");
        }

        for func in &module.functions {
            output.push_str(&render_function(func, &self.prefix));
            output.push_str("\n\n");
        }

        for cls in &module.classes {
            output.push_str(&render_class(cls, &self.prefix));
            output.push_str("\n\n");
        }

        output
    }
}

impl Default for SwiftEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::codec::CodecPlan;
    use crate::ir::types::PrimitiveType;
    use crate::render::swift::SwiftVariantPayload;

    #[test]
    fn snapshot_blittable_point() {
        let record = SwiftRecord {
            class_name: "Point".to_string(),
            fields: vec![
                SwiftField {
                    swift_name: "x".to_string(),
                    swift_type: "Double".to_string(),
                    default_expr: None,
                    codec: CodecPlan::Primitive(PrimitiveType::F64),
                    c_offset: Some(0),
                },
                SwiftField {
                    swift_name: "y".to_string(),
                    swift_type: "Double".to_string(),
                    default_expr: None,
                    codec: CodecPlan::Primitive(PrimitiveType::F64),
                    c_offset: Some(8),
                },
            ],
            is_blittable: true,
            blittable_size: Some(16),
        };
        insta::assert_snapshot!(render_record(&record));
    }

    #[test]
    fn snapshot_blittable_with_alignment_padding() {
        let record = SwiftRecord {
            class_name: "Padded".to_string(),
            fields: vec![
                SwiftField {
                    swift_name: "a".to_string(),
                    swift_type: "UInt8".to_string(),
                    default_expr: None,
                    codec: CodecPlan::Primitive(PrimitiveType::U8),
                    c_offset: Some(0),
                },
                SwiftField {
                    swift_name: "b".to_string(),
                    swift_type: "UInt32".to_string(),
                    default_expr: None,
                    codec: CodecPlan::Primitive(PrimitiveType::U32),
                    c_offset: Some(4),
                },
                SwiftField {
                    swift_name: "c".to_string(),
                    swift_type: "UInt8".to_string(),
                    default_expr: None,
                    codec: CodecPlan::Primitive(PrimitiveType::U8),
                    c_offset: Some(8),
                },
            ],
            is_blittable: true,
            blittable_size: Some(12),
        };
        insta::assert_snapshot!(render_record(&record));
    }

    #[test]
    fn snapshot_encoded_record_with_string() {
        let record = SwiftRecord {
            class_name: "User".to_string(),
            fields: vec![
                SwiftField {
                    swift_name: "id".to_string(),
                    swift_type: "Int64".to_string(),
                    default_expr: None,
                    codec: CodecPlan::Primitive(PrimitiveType::I64),
                    c_offset: None,
                },
                SwiftField {
                    swift_name: "name".to_string(),
                    swift_type: "String".to_string(),
                    default_expr: None,
                    codec: CodecPlan::String,
                    c_offset: None,
                },
            ],
            is_blittable: false,
            blittable_size: None,
        };
        insta::assert_snapshot!(render_record(&record));
    }

    #[test]
    fn snapshot_record_with_default_value() {
        let record = SwiftRecord {
            class_name: "Config".to_string(),
            fields: vec![
                SwiftField {
                    swift_name: "timeout".to_string(),
                    swift_type: "Double".to_string(),
                    default_expr: Some("30.0".to_string()),
                    codec: CodecPlan::Primitive(PrimitiveType::F64),
                    c_offset: Some(0),
                },
                SwiftField {
                    swift_name: "retries".to_string(),
                    swift_type: "Int32".to_string(),
                    default_expr: Some("3".to_string()),
                    codec: CodecPlan::Primitive(PrimitiveType::I32),
                    c_offset: Some(8),
                },
            ],
            is_blittable: true,
            blittable_size: Some(12),
        };
        insta::assert_snapshot!(render_record(&record));
    }

    #[test]
    fn snapshot_c_style_enum() {
        let e = SwiftEnum {
            name: "Status".to_string(),
            is_c_style: true,
            is_error: false,
            variants: vec![
                SwiftVariant {
                    swift_name: "active".to_string(),
                    discriminant: 0,
                    payload: SwiftVariantPayload::Unit,
                },
                SwiftVariant {
                    swift_name: "inactive".to_string(),
                    discriminant: 1,
                    payload: SwiftVariantPayload::Unit,
                },
                SwiftVariant {
                    swift_name: "pending".to_string(),
                    discriminant: 2,
                    payload: SwiftVariantPayload::Unit,
                },
            ],
            doc: None,
        };
        insta::assert_snapshot!(render_enum(&e));
    }

    #[test]
    fn snapshot_c_style_error_enum() {
        let e = SwiftEnum {
            name: "ApiError".to_string(),
            is_c_style: true,
            is_error: true,
            variants: vec![
                SwiftVariant {
                    swift_name: "notFound".to_string(),
                    discriminant: 0,
                    payload: SwiftVariantPayload::Unit,
                },
                SwiftVariant {
                    swift_name: "unauthorized".to_string(),
                    discriminant: 1,
                    payload: SwiftVariantPayload::Unit,
                },
                SwiftVariant {
                    swift_name: "serverError".to_string(),
                    discriminant: 2,
                    payload: SwiftVariantPayload::Unit,
                },
            ],
            doc: None,
        };
        insta::assert_snapshot!(render_enum(&e));
    }

    #[test]
    fn snapshot_data_enum_with_payloads() {
        let e = SwiftEnum {
            name: "Message".to_string(),
            is_c_style: false,
            is_error: false,
            variants: vec![
                SwiftVariant {
                    swift_name: "empty".to_string(),
                    discriminant: 0,
                    payload: SwiftVariantPayload::Unit,
                },
                SwiftVariant {
                    swift_name: "text".to_string(),
                    discriminant: 1,
                    payload: SwiftVariantPayload::Tuple(vec![SwiftField {
                        swift_name: "value".to_string(),
                        swift_type: "String".to_string(),
                        default_expr: None,
                        codec: CodecPlan::String,
                        c_offset: None,
                    }]),
                },
                SwiftVariant {
                    swift_name: "number".to_string(),
                    discriminant: 2,
                    payload: SwiftVariantPayload::Tuple(vec![SwiftField {
                        swift_name: "value".to_string(),
                        swift_type: "Int64".to_string(),
                        default_expr: None,
                        codec: CodecPlan::Primitive(PrimitiveType::I64),
                        c_offset: None,
                    }]),
                },
            ],
            doc: None,
        };
        insta::assert_snapshot!(render_enum(&e));
    }

    #[test]
    fn snapshot_data_enum_with_struct_payload() {
        let e = SwiftEnum {
            name: "Event".to_string(),
            is_c_style: false,
            is_error: false,
            variants: vec![
                SwiftVariant {
                    swift_name: "click".to_string(),
                    discriminant: 0,
                    payload: SwiftVariantPayload::Struct(vec![
                        SwiftField {
                            swift_name: "x".to_string(),
                            swift_type: "Int32".to_string(),
                            default_expr: None,
                            codec: CodecPlan::Primitive(PrimitiveType::I32),
                            c_offset: None,
                        },
                        SwiftField {
                            swift_name: "y".to_string(),
                            swift_type: "Int32".to_string(),
                            default_expr: None,
                            codec: CodecPlan::Primitive(PrimitiveType::I32),
                            c_offset: None,
                        },
                    ]),
                },
                SwiftVariant {
                    swift_name: "keyPress".to_string(),
                    discriminant: 1,
                    payload: SwiftVariantPayload::Struct(vec![SwiftField {
                        swift_name: "code".to_string(),
                        swift_type: "UInt32".to_string(),
                        default_expr: None,
                        codec: CodecPlan::Primitive(PrimitiveType::U32),
                        c_offset: None,
                    }]),
                },
            ],
            doc: None,
        };
        insta::assert_snapshot!(render_enum(&e));
    }
}
