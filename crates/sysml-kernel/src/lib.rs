//! Reusable, host-independent analysis of explicitly supplied SysML/KerML sources.
//!
//! Syntax and semantics are supplied by the pinned Open-MBEE libraries. This
//! facade owns input policy, source mapping, owned snapshots and query contracts.
//! It does not read files, start threads, draw diagrams, or depend on Tauri.
mod cardinality;
mod connections;
mod engine;
mod imports;
mod namespaces;
mod query;
mod source;
mod states;
mod syntax;
mod types;

pub use engine::Engine;
pub use query::{EffectiveMembers, InheritedMemberships, QueryError, SnapshotQuery};
pub use source::SourceIndex;
pub use syntax::{check_syntax, SyntaxReport};
pub use types::*;

/// Required stack for native host threads calling the upstream recursive passes.
/// The WebAssembly build reserves the same amount in its linker configuration.
pub const ANALYSIS_STACK_BYTES: usize = 16 * 1024 * 1024;

pub const SCHEMA_VERSION: u32 = 4;
pub const ENGINE_VERSION: &str = "open-mbee/0.10.2@821221767c3c+sysml-kernel.22";
pub const LIBRARY_VERSION: &str = "SysML-v2-Release/2026-03@cd99f7ca70b9";
