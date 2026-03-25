use crate::{
    ir::{
        AbiEnumField, AbiEnumPayload, AbiEnumVariant, EnumDef, EnumRepr, abi::AbiContract,
        contract::FfiContract,
    },
    render::{
        TypeMappings,
        dart::{DartEnum, DartEnumKind, DartEnumVariant, DartLibrary, NamingConvention},
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

        DartLibrary { enums }
    }

    fn lower_enum_field(&self, field: &AbiEnumField) -> super::DartEnumField {
        let field_name = super::NamingConvention::property_name(field.name.as_str());

        let dart_type = super::emit::type_expr_dart_type(&field.type_expr);

        super::DartEnumField {
            name: field_name,
            dart_type,
            wire_decode_expr: todo!(),
            wire_size_expr: todo!(),
            wire_encode_expr: todo!(),
        }
    }

    fn lower_enum_variant(
        &self,
        variant: &AbiEnumVariant,
        enum_name: &str,
        enum_kind: DartEnumKind,
    ) -> DartEnumVariant {
        let variant_name = match enum_kind {
            DartEnumKind::CStyle | DartEnumKind::Enhanced => {
                NamingConvention::property_name(variant.name.as_str())
            }
            DartEnumKind::SealedClass => format!(
                "{}{}",
                enum_name,
                NamingConvention::class_name(variant.name.as_str())
            ),
        };

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
}
