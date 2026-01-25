//! Language-specific code generation backends.
//!
//! Each backend converts the language-agnostic IR into idiomatic source code
//! for its target platform. The IR provides two inputs:
//!
//! - `FfiContract`: semantic definitions (types, functions, classes, callbacks)
//! - `LoweredContract`: transport plans (how to marshal each call across FFI)
//!
//! # Backend Implementation Guide
//!
//! Every backend follows a three-stage pipeline:
//!
//! ```text
//! 1. LOWER: FfiContract + LoweredContract → LanguageModule
//!    Convert transport plans to language-specific constructs.
//!    Map Rust types to target types (i32 → Int32, String → String, etc.)
//!    Decide naming conventions (snake_case → camelCase/PascalCase)
//!
//! 2. PLAN: LanguageModule contains all rendering decisions
//!    No more IR access needed after this stage.
//!    All type names, parameter conversions, return handling resolved.
//!
//! 3. EMIT: LanguageModule → String
//!    Pure string generation from the plan.
//!    Template-based or direct string building.
//! ```
//!
//! # What FfiContract Provides
//!
//! - `catalog`: all type definitions (records, enums, classes, callbacks, customs)
//! - `functions`: top-level exported functions
//! - `package`: crate name and version
//!
//! Use `catalog.all_records()`, `catalog.all_classes()`, etc. to iterate definitions.
//! Use `catalog.resolve_record(id)` to look up a definition by ID.
//!
//! # What LoweredContract Provides
//!
//! - `functions`: HashMap<FunctionId, CallPlan>
//! - `methods`: HashMap<(ClassId, MethodId), CallPlan>
//! - `constructors`: HashMap<(ClassId, usize), CallPlan>
//! - `callbacks`: HashMap<CallbackId, Vec<CallPlan>>
//!
//! Each `CallPlan` contains:
//! - `target`: GlobalSymbol(ffi_name) or VtableField(method_id)
//! - `params`: Vec<ParamPlan> with strategy (Direct/Buffer/Encoded/Handle/Callback)
//! - `kind`: Sync { returns } or Async { async_plan }
//!
//! # Adding a New Backend
//!
//! 1. Create `render/{lang}/plan.rs` with language-specific types
//! 2. Create `render/{lang}/lower.rs` to convert IR → plan
//! 3. Create `render/{lang}/emit.rs` to generate source code
//! 4. Export from `render/{lang}/mod.rs`
//!
//! See `render/swift/` for a complete example.

pub mod naming;
pub mod swift;

use crate::ir::{FfiContract, LoweredContract};

/// Backend interface for generating target language source code.
///
/// Implementors receive:
/// - `contract`: type definitions and function signatures (what to generate)
/// - `lowered`: marshaling strategies for each call (how to generate)
///
/// Returns language-specific output (typically `String` for source code).
///
/// # Contract
///
/// Backends must handle all constructs in `FfiContract`:
/// - Records (blittable and encoded)
/// - Enums (C-style and data-carrying)
/// - Classes (constructors, methods, static methods)
/// - Callbacks (protocol/interface with methods)
/// - Top-level functions (sync and async)
///
/// Backends must respect `LoweredContract` transport decisions:
/// - `ParamStrategy::Direct` → pass primitive as-is
/// - `ParamStrategy::Buffer` → pass pointer + length
/// - `ParamStrategy::Encoded` → serialize to wire buffer
/// - `ParamStrategy::Handle` → pass opaque pointer
/// - `ParamStrategy::Callback` → wrap in platform callback mechanism
pub trait Renderer {
    type Output;

    fn render(contract: &FfiContract, lowered: &LoweredContract) -> Self::Output;
}
