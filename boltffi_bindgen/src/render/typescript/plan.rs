#[derive(Debug, Clone)]
pub struct TsModule {
    pub module_name: String,
    pub abi_version: u32,
    pub records: Vec<TsRecord>,
    pub enums: Vec<TsEnum>,
    pub functions: Vec<TsFunction>,
    pub wasm_imports: Vec<TsWasmImport>,
}

#[derive(Debug, Clone)]
pub struct TsRecord {
    pub name: String,
    pub fields: Vec<TsField>,
    pub is_blittable: bool,
    pub wire_size: Option<usize>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TsField {
    pub name: String,
    pub ts_type: String,
    pub wire_decode_expr: String,
    pub wire_encode_expr: String,
    pub wire_size_expr: String,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TsEnum {
    pub name: String,
    pub variants: Vec<TsVariant>,
    pub kind: TsEnumKind,
    pub doc: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TsEnumKind {
    CStyle,
    Data,
}

impl TsEnum {
    pub fn is_c_style(&self) -> bool {
        matches!(self.kind, TsEnumKind::CStyle)
    }
}

#[derive(Debug, Clone)]
pub struct TsVariant {
    pub name: String,
    pub discriminant: i64,
    pub fields: Vec<TsVariantField>,
    pub doc: Option<String>,
}

impl TsVariant {
    pub fn is_unit(&self) -> bool {
        self.fields.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct TsVariantField {
    pub name: String,
    pub ts_type: String,
    pub wire_decode_expr: String,
    pub wire_encode_expr: String,
    pub wire_size_expr: String,
}

#[derive(Debug, Clone)]
pub struct TsFunction {
    pub name: String,
    pub ffi_name: String,
    pub params: Vec<TsParam>,
    pub return_type: Option<String>,
    pub return_abi: TsReturnAbi,
    pub decode_expr: String,
    pub throws: bool,
    pub err_type: String,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TsParam {
    pub name: String,
    pub ts_type: String,
    pub conversion: TsParamConversion,
}

impl TsParam {
    pub fn wrapper_code(&self) -> Option<String> {
        match &self.conversion {
            TsParamConversion::Direct => None,
            TsParamConversion::String => {
                Some(format!("const {}_alloc = _module.allocString({});", self.name, self.name))
            }
            TsParamConversion::WireEncoded { encode_expr, size_expr } => {
                Some(format!(
                    "const {name}_writer = _module.allocWriter({size_expr});\n  {encode_expr};\n  const {name}_ptr = {name}_writer.ptr;",
                    name = self.name,
                    size_expr = size_expr,
                    encode_expr = encode_expr.replace("writer", &format!("{}_writer", self.name)).replace("value", &self.name),
                ))
            }
        }
    }

    pub fn ffi_args(&self) -> Vec<String> {
        match &self.conversion {
            TsParamConversion::Direct => vec![self.name.clone()],
            TsParamConversion::String => {
                vec![
                    format!("{}_alloc.ptr", self.name),
                    format!("{}_alloc.len", self.name),
                ]
            }
            TsParamConversion::WireEncoded { .. } => {
                vec![format!("{}_ptr", self.name)]
            }
        }
    }

    pub fn cleanup_code(&self) -> Option<String> {
        match &self.conversion {
            TsParamConversion::Direct => None,
            TsParamConversion::String => {
                Some(format!("_module.freeAlloc({}_alloc);", self.name))
            }
            TsParamConversion::WireEncoded { .. } => {
                Some(format!("_module.freeWriter({}_writer);", self.name))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TsParamConversion {
    Direct,
    String,
    WireEncoded {
        encode_expr: String,
        size_expr: String,
    },
}

#[derive(Debug, Clone)]
pub enum TsReturnAbi {
    Void,
    Direct { ts_cast: String },
    WireEncoded,
}

impl TsReturnAbi {
    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn is_direct(&self) -> bool {
        matches!(self, Self::Direct { .. })
    }

    pub fn is_wire_encoded(&self) -> bool {
        matches!(self, Self::WireEncoded)
    }
}

#[derive(Debug, Clone)]
pub struct TsWasmImport {
    pub ffi_name: String,
    pub params: Vec<TsWasmParam>,
    pub return_wasm_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TsWasmParam {
    pub name: String,
    pub wasm_type: String,
}
