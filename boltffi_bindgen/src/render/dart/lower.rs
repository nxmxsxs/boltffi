use crate::{
    ir::{abi::AbiContract, contract::FfiContract},
    render::TypeMappings,
};

pub struct DartLowerer<'a> {
    ffi: &'a FfiContract,
    abi: &'a AbiContract,
    package_name: String,
    module_name: String,
    type_mappings: TypeMappings,
}
