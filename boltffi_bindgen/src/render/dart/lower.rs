use crate::{
    ir::{
        AbiCall, AbiEnumField, AbiEnumPayload, AbiEnumVariant, AbiParam, AbiRecord, AbiType,
        CallId, EnumDef, EnumRepr, FieldDef, FieldName, FieldReadOp, FunctionDef, OffsetExpr,
        ParamRole, ReadOp, ReadSeq, RecordDef, RecordId, WriteOp, WriteSeq, abi::AbiContract,
        contract::FfiContract,
    },
    render::{
        TypeMappings,
        dart::{
            DartBlittableField, DartBlittableLayout, DartEnum, DartEnumKind, DartEnumVariant,
            DartLibrary, DartNative, DartNativeFunction, DartNativeFunctionParam, DartNativeType,
            DartRecord, DartRecordField, NamingConvention, emit,
        },
    },
};

pub struct DartLowerer<'a> {
    ffi: &'a FfiContract,
    abi: &'a AbiContract,
    package_name: String,
    module_name: String,
    type_mappings: TypeMappings,
}

impl<'a> DartLowerer<'a> {
    pub fn new(
        ffi: &'a FfiContract,
        abi: &'a AbiContract,
        package_name: String,
        module_name: String,
    ) -> Self {
        Self {
            ffi,
            abi,
            package_name,
            module_name,
            type_mappings: TypeMappings::new(),
        }
    }

    pub fn library(&self) -> DartLibrary {
        let prefix = boltffi_ffi_rules::naming::ffi_prefix().to_string();

        let native_functions = self
            .ffi
            .functions
            .iter()
            .map(|f| {
                let abi_call = self
                    .abi
                    .calls
                    .iter()
                    .find(|c| match &c.id {
                        CallId::Function(id) => id == &f.id,
                        _ => false,
                    })
                    .unwrap();
                self.lower_native_function(abi_call)
            })
            .collect();

        let enums = self
            .ffi
            .catalog
            .all_enums()
            .map(|e| self.lower_enum(e))
            .collect();

        let records = self
            .ffi
            .catalog
            .all_records()
            .map(|r| self.lower_record(r))
            .collect();

        DartLibrary {
            enums,
            records,
            native: DartNative {
                functions: native_functions,
            },
        }
    }

    fn lower_enum_field(&self, field: &AbiEnumField) -> super::DartEnumField {
        let field_name = super::NamingConvention::property_name(field.name.as_str());

        let dart_type = super::emit::type_expr_dart_type(&field.type_expr);

        super::DartEnumField {
            name: field_name,
            dart_type,
            wire_decode_expr: super::emit::emit_reader_read(&field.decode),
            wire_size_expr: super::emit::emit_size_expr(&field.decode.size),
            wire_encode_expr: super::emit::emit_write_expr(&field.encode, "writer"),
        }
    }

    fn lower_enum_variant(
        &self,
        variant: &AbiEnumVariant,
        enum_name: &str,
        enum_kind: DartEnumKind,
    ) -> DartEnumVariant {
        let variant_name = NamingConvention::property_name(variant.name.as_str());
        let variant_class_name = format!(
            "{}{}",
            enum_name,
            NamingConvention::class_name(variant.name.as_str())
        );

        let fields = match &variant.payload {
            AbiEnumPayload::Unit => Vec::new(),
            AbiEnumPayload::Tuple(abi_enum_fields) | AbiEnumPayload::Struct(abi_enum_fields) => {
                abi_enum_fields
                    .iter()
                    .map(|f| self.lower_enum_field(f))
                    .collect()
            }
        };

        DartEnumVariant {
            name: variant_name,
            class_name: variant_class_name,
            tag: variant.discriminant,
            fields,
        }
    }

    fn lower_enum(&self, enum_def: &EnumDef) -> DartEnum {
        let enum_name = NamingConvention::class_name(enum_def.id.as_str());

        let abi_enum = self
            .abi
            .enums
            .iter()
            .find(|en| en.id == enum_def.id)
            .unwrap();

        let enum_kind = if abi_enum.is_c_style {
            DartEnumKind::Enhanced
        } else {
            DartEnumKind::SealedClass
        };

        let tag_type = match &enum_def.repr {
            EnumRepr::CStyle { tag_type, .. } | EnumRepr::Data { tag_type, .. } => {
                super::emit::primitive_dart_type(*tag_type)
            }
        };

        let enum_variants = abi_enum
            .variants
            .iter()
            .map(|v| self.lower_enum_variant(v, &enum_name, enum_kind))
            .collect();

        DartEnum {
            name: enum_name,
            kind: enum_kind,
            tag_type,
            variants: enum_variants,
        }
    }

    fn abi_record_for(&self, record_id: &RecordId) -> Option<&AbiRecord> {
        self.abi
            .records
            .iter()
            .find(|record| record.id == *record_id)
    }

    fn record_field_read_seq(
        &self,
        abi_record: &AbiRecord,
        field_name: &FieldName,
    ) -> Option<ReadSeq> {
        match abi_record.decode_ops.ops.first() {
            Some(ReadOp::Record { fields, .. }) => fields
                .iter()
                .find(|field| field.name == *field_name)
                .map(|field| field.seq.clone()),
            _ => None,
        }
    }

    fn record_field_write_seq(
        &self,
        abi_record: &AbiRecord,
        field_name: &FieldName,
    ) -> Option<WriteSeq> {
        match abi_record.encode_ops.ops.first() {
            Some(WriteOp::Record { fields, .. }) => fields
                .iter()
                .find(|field| field.name == *field_name)
                .map(|field| field.seq.clone()),
            _ => None,
        }
    }

    fn lower_record_field(&self, field: &FieldDef, abi_record: &AbiRecord) -> DartRecordField {
        let record_field_write_seq = self
            .record_field_write_seq(abi_record, &field.name)
            .unwrap();
        let record_field_read_seq = self.record_field_read_seq(abi_record, &field.name).unwrap();

        DartRecordField {
            name: field.name.to_string(),
            offset: 0,
            dart_type: super::emit::type_expr_dart_type(&field.type_expr),
            wire_decode_expr: super::emit::emit_reader_read(&record_field_read_seq),
            wire_encode_expr: super::emit::emit_write_expr(&record_field_write_seq, "writer"),
        }
    }

    fn lower_record_blittable_field(&self, field: &FieldReadOp) -> DartBlittableField {
        let (primitive, offset) = match field.seq.ops.first() {
            Some(ReadOp::Primitive { primitive, offset }) => (*primitive, offset),
            _ => unreachable!(),
        };
        let offset = match offset {
            OffsetExpr::Base => 0,
            OffsetExpr::BasePlus(offset) => *offset,
            _ => unreachable!(),
        };
        let name = NamingConvention::property_name(field.name.as_str());
        // let const_name = NamingConvention::enum_constant_name(field.name.as_str());
        DartBlittableField {
            name,
            offset,
            native_type: DartNativeType::from_primitive(&primitive),
            const_name: String::new(),
            // decode_expr: java_blittable_decode_expr(primitive, &const_name),
            // encode_expr: java_blittable_encode_expr(primitive, &const_name, &name),
            decode_expr: String::new(),
            encode_expr: String::new(),
        }
    }

    fn lower_record_blittable_layout(&self, abi_record: &AbiRecord) -> DartBlittableLayout {
        let fields = match abi_record.decode_ops.ops.first() {
            Some(ReadOp::Record { fields, .. }) => fields
                .iter()
                .map(|f| self.lower_record_blittable_field(f))
                .collect(),
            _ => unreachable!(),
        };

        DartBlittableLayout {
            fields,
            struct_size: abi_record
                .size
                .expect("record.is_blittable <=> size != None"),
        }
    }

    fn lower_record(&self, record: &RecordDef) -> DartRecord {
        let name = NamingConvention::class_name(record.id.as_str());

        let abi_record = self.abi_record_for(&record.id).unwrap();

        let fields = record
            .fields
            .iter()
            .map(|f| self.lower_record_field(f, abi_record))
            .collect();

        let blittable_layout = abi_record
            .is_blittable
            .then(|| self.lower_record_blittable_layout(abi_record));

        DartRecord {
            name,
            fields,
            blittable_layout,
        }
    }

    fn lower_native_function_param(&self, abi_param: &AbiParam) -> DartNativeFunctionParam {
        DartNativeFunctionParam {
            name: abi_param.name.to_string(),
            native_type: DartNativeType::from_abi_param(abi_param),
        }
    }

    fn lower_native_function(&self, abi_call: &AbiCall) -> DartNativeFunction {
        let symbol = abi_call.symbol.to_string();

        let params = abi_call
            .params
            .iter()
            .map(|p| self.lower_native_function_param(p))
            .collect();

        let is_not_leaf = abi_call
            .params
            .iter()
            .any(|p| matches!(p.abi_type, AbiType::InlineCallbackFn { .. }));

        DartNativeFunction {
            symbol,
            params,
            return_type: DartNativeType::abi_call_return_type(abi_call),
            is_leaf: !is_not_leaf,
        }
    }
}
