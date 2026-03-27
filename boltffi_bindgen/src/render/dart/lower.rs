use crate::{
    ir::{
        AbiEnumField, AbiEnumPayload, AbiEnumVariant, AbiRecord, EnumDef, EnumRepr, FieldDef,
        FieldName, ReadOp, ReadSeq, RecordDef, RecordId, WriteOp, WriteSeq, abi::AbiContract,
        contract::FfiContract,
    },
    render::{
        TypeMappings,
        dart::{
            DartEnum, DartEnumKind, DartEnumVariant, DartLibrary, DartRecord, DartRecordField,
            NamingConvention,
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

        DartLibrary { enums, records }
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

    fn lower_record(&self, record: &RecordDef) -> DartRecord {
        let name = NamingConvention::class_name(record.id.as_str());

        let abi_record = self.abi_record_for(&record.id).unwrap();

        let fields = record
            .fields
            .iter()
            .map(|f| self.lower_record_field(f, abi_record))
            .collect();

        DartRecord { name, fields }
    }
}
