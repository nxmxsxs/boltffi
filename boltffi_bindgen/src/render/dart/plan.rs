pub struct DartLibrary {
    pub enums: Vec<DartEnum>,
    pub records: Vec<DartRecord>,
}

#[derive(Clone, Copy)]
pub enum DartEnumKind {
    CStyle,
    Enhanced,
    SealedClass,
}

pub struct DartEnumField {
    pub name: String,
    pub dart_type: String,
    pub wire_decode_expr: String,
    pub wire_size_expr: String,
    pub wire_encode_expr: String,
}

pub struct DartEnumVariant {
    pub name: String,
    pub tag: i128,
    pub fields: Vec<DartEnumField>,
}

pub struct DartEnum {
    pub name: String,
    pub kind: DartEnumKind,
    pub tag_type: String,
    pub variants: Vec<DartEnumVariant>,
}

pub struct DartRecordField {
    pub name: String,
    pub offset: usize,
    pub dart_type: String,
    pub wire_decode_expr: String,
    pub wire_encode_expr: String,
}

pub struct DartRecord {
    pub name: String,
    pub fields: Vec<DartRecordField>,
}
