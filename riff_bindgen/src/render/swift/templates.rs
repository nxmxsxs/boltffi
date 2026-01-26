use askama::Template;

use super::plan::{
    SwiftCallback, SwiftCallMode, SwiftClass, SwiftEnum, SwiftField, SwiftFunction, SwiftRecord,
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

    fn blittable_point_record() -> SwiftRecord {
        SwiftRecord {
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
        }
    }

    fn encoded_user_record() -> SwiftRecord {
        SwiftRecord {
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
        }
    }

    fn c_style_status_enum() -> SwiftEnum {
        SwiftEnum {
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
        }
    }

    #[test]
    fn blittable_record_generates_fixed_size_decode() {
        let record = blittable_point_record();
        let output = render_record(&record);
        
        assert!(output.contains("public struct Point"), "Should declare struct Point");
        assert!(output.contains("public var x: Double"), "Should have x field");
        assert!(output.contains("public var y: Double"), "Should have y field");
        assert!(output.contains("static var isBlittable: Bool { true }"), "Blittable should be true");
        assert!(output.contains("func wireEncodedSize() -> Int { 16 }"), "Size should be fixed 16");
    }

    #[test]
    fn blittable_record_decode_uses_offsets() {
        let record = blittable_point_record();
        let output = render_record(&record);
        
        assert!(output.contains("wire.readF64(at: offset)"), "x should read at offset");
        assert!(output.contains("wire.readF64(at: offset + 8)"), "y should read at offset + 8");
        assert!(output.contains("), 16)"), "Should return size 16");
    }

    #[test]
    fn encoded_record_generates_variable_size_decode() {
        let record = encoded_user_record();
        let output = render_record(&record);
        
        assert!(output.contains("public struct User"), "Should declare struct User");
        assert!(output.contains("static var isBlittable: Bool { false }"), "Blittable should be false");
        assert!(output.contains("var pos = offset"), "Should track position");
        assert!(output.contains("pos - offset"), "Should return consumed size");
    }

    #[test]
    fn c_style_enum_generates_int32_raw_value() {
        let e = c_style_status_enum();
        let output = render_enum(&e);
        
        assert!(output.contains("public enum Status: Int32"), "Should be Int32 raw value");
        assert!(output.contains("case active = 0"), "active should be 0");
        assert!(output.contains("case inactive = 1"), "inactive should be 1");
        assert!(output.contains("case pending = 2"), "pending should be 2");
        assert!(output.contains("CaseIterable"), "C-style enums are CaseIterable");
    }

    #[test]
    fn c_style_enum_decode_reads_i32() {
        let e = c_style_status_enum();
        let output = render_enum(&e);
        
        assert!(output.contains("wire.readI32(at: offset)"), "Should read i32");
        assert!(output.contains(", 4)"), "Size should be 4 bytes");
    }

    #[test]
    fn error_enum_conforms_to_error_protocol() {
        let mut e = c_style_status_enum();
        e.is_error = true;
        let output = render_enum(&e);
        
        assert!(output.contains("Error"), "Should conform to Error protocol");
    }

    #[test]
    fn record_init_has_all_parameters() {
        let record = blittable_point_record();
        let output = render_record(&record);
        
        assert!(output.contains("public init(x: Double, y: Double)"), "Init should have both params");
    }

    #[test]
    fn record_with_default_value() {
        let mut record = blittable_point_record();
        record.fields[0].default_expr = Some("0.0".to_string());
        let output = render_record(&record);
        
        assert!(output.contains("x: Double = 0.0"), "Should have default value");
    }

    fn data_value_enum() -> SwiftEnum {
        SwiftEnum {
            name: "Value".to_string(),
            is_c_style: false,
            is_error: false,
            variants: vec![
                SwiftVariant {
                    swift_name: "none".to_string(),
                    discriminant: 0,
                    payload: SwiftVariantPayload::Unit,
                },
                SwiftVariant {
                    swift_name: "integer".to_string(),
                    discriminant: 1,
                    payload: SwiftVariantPayload::Tuple(vec![
                        SwiftField {
                            swift_name: "0".to_string(),
                            swift_type: "Int64".to_string(),
                            default_expr: None,
                            codec: CodecPlan::Primitive(PrimitiveType::I64),
                            c_offset: None,
                        },
                    ]),
                },
                SwiftVariant {
                    swift_name: "text".to_string(),
                    discriminant: 2,
                    payload: SwiftVariantPayload::Tuple(vec![
                        SwiftField {
                            swift_name: "0".to_string(),
                            swift_type: "String".to_string(),
                            default_expr: None,
                            codec: CodecPlan::String,
                            c_offset: None,
                        },
                    ]),
                },
            ],
            doc: None,
        }
    }

    #[test]
    fn data_enum_generates_associated_values() {
        let e = data_value_enum();
        let output = render_enum(&e);
        
        assert!(output.contains("public enum Value"), "Should declare enum Value\n{}", output);
        assert!(output.contains("case none"), "Unit variant should have no parens\n{}", output);
        assert!(output.contains("case integer(Int64)") || output.contains("case integer(0: Int64)"), 
            "Single tuple should have type\n{}", output);
        assert!(output.contains("case text(String)") || output.contains("case text(0: String)"), 
            "String variant should have String type\n{}", output);
    }

    #[test]
    fn data_enum_decode_uses_tag_switch() {
        let e = data_value_enum();
        let output = render_enum(&e);
        
        assert!(output.contains("let tag = wire.readI32(at: offset)"), "Should read tag first");
        assert!(output.contains("var pos = offset + 4"), "Should advance past tag");
        assert!(output.contains("switch tag"), "Should switch on tag");
        assert!(output.contains("case 0:"), "Should have case for discriminant 0");
        assert!(output.contains("case 1:"), "Should have case for discriminant 1");
        assert!(output.contains("case 2:"), "Should have case for discriminant 2");
    }

    #[test]
    fn data_enum_encode_writes_discriminant() {
        let e = data_value_enum();
        let output = render_enum(&e);
        
        assert!(output.contains("data.appendI32(0)"), "Should write discriminant 0");
        assert!(output.contains("data.appendI32(1)"), "Should write discriminant 1");
        assert!(output.contains("data.appendI32(2)"), "Should write discriminant 2");
    }

    #[test]
    fn data_enum_is_not_blittable() {
        let e = data_value_enum();
        let output = render_enum(&e);
        
        assert!(output.contains("static var isBlittable: Bool { false }"), "Data enum should not be blittable");
    }
}
