mod codec;
mod lower;
mod plan;
mod templates;

pub use lower::SwiftLowerer;
pub use templates::{
    render_enum, render_record, EnumCStyleTemplate, EnumDataTemplate, RecordTemplate,
    SwiftEmitter,
};
pub use plan::{
    SwiftCallback, SwiftCallbackMethod, SwiftClass, SwiftConstructor, SwiftConversion, SwiftEnum,
    SwiftField, SwiftFunction, SwiftMethod, SwiftModule, SwiftParam, SwiftRecord, SwiftReturn,
    SwiftVariant, SwiftVariantPayload,
};
