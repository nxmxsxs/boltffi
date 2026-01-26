mod codec;
mod lower;
mod plan;
mod templates;

pub use lower::SwiftLowerer;
pub use plan::{
    SwiftCallback, SwiftCallbackMethod, SwiftClass, SwiftConstructor, SwiftConversion, SwiftEnum,
    SwiftField, SwiftFunction, SwiftMethod, SwiftModule, SwiftParam, SwiftRecord, SwiftReturn,
    SwiftVariant, SwiftVariantPayload,
};
pub use templates::{
    CallbackTemplate, EnumCStyleTemplate, EnumDataTemplate, RecordTemplate, SwiftEmitter,
    render_callback, render_enum, render_record,
};
