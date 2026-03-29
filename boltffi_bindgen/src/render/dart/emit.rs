use askama::Template as _;

use crate::ir::types::{PrimitiveType, TypeExpr};
use crate::ir::{
    AbiCall, AbiContract, AbiParam, AbiType, BuiltinId, EnumLayout, ErrorTransport, FfiContract,
    Mutability, ParamRole, ReadOp, ReadSeq, ReturnShape, SizeExpr, SpanContent, Transport,
    ValueExpr, VecLayout, WriteOp, WriteSeq,
};
use crate::render::dart::NamingConvention;
use crate::render::dart::lower::DartLowerer;
use crate::render::dart::templates::{
    EnhancedEnumTemplate, NativeFunctionsTemplate, NativeRecordTemplate, PreludeTemplate,
    RecordTemplate, SealedClassEnumTemplate,
};

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

        output.push_str(PreludeTemplate {}.render().unwrap().as_str());
        output.push('\n');
        output.push('\n');

        for r in &library.records {
            if let Some(layout) = &r.blittable_layout {
                output.push_str(
                    NativeRecordTemplate {
                        layout,
                        name: &r.name,
                    }
                    .render()
                    .unwrap()
                    .as_str(),
                );
                output.push('\n');
            }
        }

        output.push('\n');

        output.push_str(
            NativeFunctionsTemplate {
                cfuncs: &library.native.functions,
            }
            .render()
            .unwrap()
            .as_str(),
        );
        output.push('\n');

        for r in &library.records {
            output.push_str(RecordTemplate { record: r }.render().unwrap().as_str());
            output.push('\n');
        }

        for en in &library.enums {
            let src = match en.kind {
                super::DartEnumKind::CStyle | super::DartEnumKind::Enhanced => {
                    EnhancedEnumTemplate { dart_enum: en }.render().unwrap()
                }
                super::DartEnumKind::SealedClass => {
                    SealedClassEnumTemplate { dart_enum: en }.render().unwrap()
                }
            };

            output.push_str(&src);
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

pub fn primitive_read_method(primitive: PrimitiveType) -> &'static str {
    match primitive {
        PrimitiveType::Bool => "readBool",
        PrimitiveType::I8 => "readI8",
        PrimitiveType::U8 => "readU8",
        PrimitiveType::I16 => "readI16",
        PrimitiveType::U16 => "readU16",
        PrimitiveType::I32 => "readI32",
        PrimitiveType::U32 => "readU32",
        PrimitiveType::I64 | PrimitiveType::ISize => "readI64",
        PrimitiveType::U64 | PrimitiveType::USize => "readU64",
        PrimitiveType::F32 => "readF32",
        PrimitiveType::F64 => "readF64",
    }
}

fn emit_reader_vec(element_type: &TypeExpr, element: &ReadSeq, layout: &VecLayout) -> String {
    match layout {
        VecLayout::Blittable { .. } => match element_type {
            TypeExpr::Primitive(primitive) => {
                let method = match primitive {
                    PrimitiveType::U8 | PrimitiveType::Bool => "readUint8List",
                    PrimitiveType::I8 => "readInt8List",
                    PrimitiveType::I16 => "readInt16List",
                    PrimitiveType::U16 => "readUint16List",
                    PrimitiveType::I32 => "readInt32List",
                    PrimitiveType::U32 => "readUint32List",
                    PrimitiveType::U64 | PrimitiveType::USize => "readUint64List",
                    PrimitiveType::I64 | PrimitiveType::ISize => "readInt64List",
                    PrimitiveType::F32 => "readFloat32List",
                    PrimitiveType::F64 => "readFloat64List",
                };
                format!("reader.{}()", method)
            }
            _ => {
                let inner = emit_reader_read(element);
                format!("reader.readList((reader) => {})", inner)
            }
        },
        VecLayout::Encoded => {
            let inner = emit_reader_read(element);
            format!("reader.readList((reader) => {})", inner)
        }
    }
}

pub fn emit_reader_read(seq: &ReadSeq) -> String {
    let op = seq.ops.first().expect("read ops");
    match op {
        ReadOp::Primitive { primitive, .. } => {
            format!("reader.{}()", primitive_read_method(*primitive))
        }
        ReadOp::String { .. } => "reader.readString()".to_string(),
        ReadOp::Bytes { .. } => "reader.readUint8List()".to_string(),
        ReadOp::Record { id, .. } => {
            format!("{}.decode(reader)", render_type_name(id.as_str()))
        }
        ReadOp::Enum { id, layout, .. } => match layout {
            EnumLayout::CStyle {
                tag_type,
                tag_strategy,
                is_error: false,
            } => {
                format!(
                    "{}.fromValue(reader.{}())",
                    render_type_name(id.as_str()),
                    primitive_read_method(*tag_type),
                )
            }
            EnumLayout::CStyle { is_error: true, .. }
            | EnumLayout::Data { .. }
            | EnumLayout::Recursive => {
                format!("{}.decode(reader)", render_type_name(id.as_str()))
            }
        },
        ReadOp::Option { some, .. } => {
            let inner = emit_reader_read(some);
            format!("reader.readOptional((reader) => {})", inner)
        }
        ReadOp::Vec {
            element_type,
            element,
            layout,
            ..
        } => emit_reader_vec(element_type, element, layout),
        ReadOp::Result { ok, err, .. } => {
            let ok_expr = emit_reader_read(ok);
            let err_expr = emit_reader_read(err);
            todo!()
        }
        ReadOp::Builtin { id, .. } => match id.as_str() {
            "Duration" => "reader.readDuration()".to_string(),
            "SystemTime" => "reader.readInstant()".to_string(),
            "Uuid" => "reader.readUuid()".to_string(),
            "Url" => "reader.readUri()".to_string(),
            _ => "reader.readString()".to_string(),
        },
        ReadOp::Custom { id, .. } => {
            format!("{}.decode(reader)", render_type_name(id.as_str()))
        }
    }
}

pub fn render_value(expr: &ValueExpr) -> String {
    match expr {
        ValueExpr::Instance => String::new(),
        ValueExpr::Var(name) => name.clone(),
        ValueExpr::Named(name) => NamingConvention::property_name(name),
        ValueExpr::Field(parent, field) => {
            let parent_str = render_value(parent);
            let field_str = NamingConvention::property_name(field.as_str());
            if parent_str.is_empty() {
                field_str
            } else {
                format!("{}.{}", parent_str, field_str)
            }
        }
    }
}

fn emit_vec_size(value: &str, inner: &SizeExpr, layout: &VecLayout) -> String {
    match layout {
        VecLayout::Blittable { .. } => {
            format!("(4 + {}.length * {})", value, emit_size_expr(inner))
        }
        VecLayout::Encoded => {
            todo!()
        }
    }
}

fn emit_builtin_size(id: &BuiltinId, value: &str) -> String {
    if id.as_str() == "Url" {
        format!("{}.toString().length * 3", value)
    } else {
        format!("{}.wireEncodedSize()", value)
    }
}

pub fn emit_size_expr(size: &SizeExpr) -> String {
    match size {
        SizeExpr::Fixed(value) => value.to_string(),
        SizeExpr::Runtime => "0".to_string(),
        SizeExpr::StringLen(value) => format!("{}.length", render_value(value)),
        SizeExpr::BytesLen(value) => format!("{}.length", render_value(value)),
        SizeExpr::ValueSize(value) => render_value(value),
        SizeExpr::WireSize { value, .. } => format!("{}.wireEncodedSize()", render_value(value)),
        SizeExpr::BuiltinSize { id, value } => emit_builtin_size(id, &render_value(value)),
        SizeExpr::Sum(parts) => {
            let rendered = parts
                .iter()
                .map(emit_size_expr)
                .collect::<Vec<_>>()
                .join(" + ");
            format!("({})", rendered)
        }
        SizeExpr::OptionSize { value, inner } => {
            let inner_expr = emit_size_expr(inner);
            format!(
                "(switch ({} == null) {{ true => 1, false => 1 + {} }})",
                render_value(value),
                inner_expr
            )
        }
        SizeExpr::VecSize {
            value,
            inner,
            layout,
        } => emit_vec_size(&render_value(value), inner, layout),
        SizeExpr::ResultSize { value, ok, err } => {
            let v = render_value(value);
            let ok_expr = emit_size_expr(ok);
            let err_expr = emit_size_expr(err);
            todo!()
        }
    }
}

fn emit_write_primitive(primitive: PrimitiveType, value: &str) -> String {
    match primitive {
        PrimitiveType::Bool => format!("writer.writeBool({})", value),
        PrimitiveType::I8 => format!("writer.writeI8({})", value),
        PrimitiveType::U8 => format!("writer.writeU8({})", value),
        PrimitiveType::I16 => format!("writer.writeI16({})", value),
        PrimitiveType::U16 => format!("writer.writeU16({})", value),
        PrimitiveType::I32 => format!("writer.writeI32({})", value),
        PrimitiveType::U32 => format!("writer.writeU32({})", value),
        PrimitiveType::I64 | PrimitiveType::ISize => format!("writer.writeI64({})", value),
        PrimitiveType::U64 | PrimitiveType::USize => format!("writer.writeU64({})", value),
        PrimitiveType::F32 => format!("writer.writeF32({})", value),
        PrimitiveType::F64 => format!("writer.writeF64({})", value),
    }
}

fn emit_write_builtin(id: &BuiltinId, value: &str) -> String {
    match id.as_str() {
        "Duration" => format!("writer.writeDuration({})", value),
        "SystemTime" => format!("writer.writeInstant({})", value),
        "Uuid" => format!("writer.writeUuid({})", value),
        "Url" => format!("writer.writeUri({})", value),
        _ => format!("writer.writeString({})", value),
    }
}

pub fn primitive_write_method(primitive: PrimitiveType) -> &'static str {
    match primitive {
        PrimitiveType::Bool => "writeBool",
        PrimitiveType::I8 => "writeI8",
        PrimitiveType::U8 => "writeU8",
        PrimitiveType::I16 => "writeI16",
        PrimitiveType::U16 => "writeU16",
        PrimitiveType::I32 => "writeI32",
        PrimitiveType::U32 => "writeU32",
        PrimitiveType::I64 | PrimitiveType::ISize => "writeI64",
        PrimitiveType::U64 | PrimitiveType::USize => "writeU64",
        PrimitiveType::F32 => "writeF32",
        PrimitiveType::F64 => "writeF64",
    }
}

fn enum_tag_write_expr(tag_type: PrimitiveType, value_expr: &str) -> String {
    let write_method = primitive_write_method(tag_type);
    format!("writer.{write_method}({value_expr})")
}

fn write_seq_dart_type(seq: &WriteSeq) -> String {
    match seq.ops.first() {
        Some(WriteOp::Primitive { primitive, .. }) => {
            type_expr_dart_type(&TypeExpr::Primitive(*primitive))
        }
        Some(WriteOp::String { .. }) => "String".to_string(),
        Some(WriteOp::Bytes { .. }) => "Uint8List".to_string(),
        Some(WriteOp::Builtin { id, .. }) => type_expr_dart_type(&TypeExpr::Builtin(id.clone())),
        Some(WriteOp::Record { id, .. }) => render_type_name(id.as_str()),
        Some(WriteOp::Enum { id, .. }) => render_type_name(id.as_str()),
        Some(WriteOp::Custom { id, .. }) => render_type_name(id.as_str()),
        Some(WriteOp::Vec { element_type, .. }) => {
            type_expr_dart_type(&TypeExpr::Vec(Box::new(element_type.clone())))
        }
        Some(WriteOp::Option { some, .. }) => format!("{}?", write_seq_dart_type(some)),
        Some(WriteOp::Result { ok, err, .. }) => format!(
            "BoltFFIResult<{}, {}>",
            write_seq_dart_type(ok),
            write_seq_dart_type(err)
        ),
        _ => "dynamic".to_string(),
    }
}

fn emit_write_vec(
    value: &str,
    element_type: &TypeExpr,
    element: &WriteSeq,
    layout: &VecLayout,
) -> String {
    // match layout {
    //     VecLayout::Blittable { .. } => match element_type {
    //         TypeExpr::Primitive(_) => format!("wire.writePrimitiveList({})", value),
    //         TypeExpr::Record(id) => format!(
    //             "wire.writeU32({}.size.toUInt()); {}Writer.writeAllToWire(wire, {})",
    //             value,
    //             id.as_str(),
    //             value
    //         ),
    //         _ => {
    //             let inner = emit_write_expr(element, "writer");
    //             format!(
    //                 "wire.writeU32({}.size.toUInt()); {}.forEach {{ item -> {} }}",
    //                 value, value, inner
    //             )
    //         }
    //     },
    //     VecLayout::Encoded => {
    //         let inner = emit_write_expr(element, "writer");
    //         format!(
    //             "wire.writeU32({}.size.toUInt()); {}.forEach {{ item -> {} }}",
    //             value, value, inner
    //         )
    //     }
    // }
    todo!()
}

pub fn emit_write_expr(seq: &WriteSeq, writer_name: &str) -> String {
    let op = seq.ops.first().expect("write ops");
    match op {
        WriteOp::Primitive { primitive, value } => {
            emit_write_primitive(*primitive, &render_value(value))
        }
        WriteOp::String { value } => {
            format!("{}.writeString({})", writer_name, render_value(value))
        }
        WriteOp::Bytes { value } => format!("{}.writeBytes({})", writer_name, render_value(value)),
        WriteOp::Option { value, some } => {
            let inner = emit_write_expr(some, writer_name);

            format!(
                "if ({value} == null) {{ {writer_name}.writeI32(0); }} else {{ {writer_name}.writeI32(1);{inner}; }}",
                value = render_value(value),
            )
        }
        WriteOp::Vec {
            value,
            element_type,
            element,
            layout,
        } => emit_write_vec(&render_value(value), element_type, element, layout),
        WriteOp::Record { value, .. } => {
            format!("{}.wireEncodeTo({writer_name})", render_value(value))
        }
        WriteOp::Enum { value, layout, .. } => match layout {
            EnumLayout::CStyle {
                tag_type,
                tag_strategy: _,
                is_error: false,
            } => enum_tag_write_expr(*tag_type, &format!("{}.value", render_value(value))),
            EnumLayout::CStyle { is_error: true, .. }
            | EnumLayout::Data { .. }
            | EnumLayout::Recursive => {
                format!("{}.wireEncodeTo({writer_name})", render_value(value))
            }
        },
        WriteOp::Result { value, ok, err } => {
            let v = render_value(value);
            let ok_expr = emit_write_expr(ok, writer_name);
            let err_expr = emit_write_expr(err, writer_name);
            let ok_type = write_seq_dart_type(ok);
            let err_type = write_seq_dart_type(err);
            // format!(
            //     "when ({}) {{ is BoltFFIResult.Ok<*> -> {{ wire.writeU8(0u); val okVal = boltffiUnsafeCast<{}>({}.value); {} }} is BoltFFIResult.Err<*> -> {{ wire.writeU8(1u); val errVal = boltffiUnsafeCast<{}>({}.error); {} }} }}",
            //     v, ok_type, v, ok_expr, err_type, v, err_expr
            // )
            todo!()
        }
        WriteOp::Builtin { id, value } => emit_write_builtin(id, &render_value(value)),
        WriteOp::Custom { value, .. } => {
            format!("{}.wireEncodeTo({writer_name})", render_value(value))
        }
    }
}
