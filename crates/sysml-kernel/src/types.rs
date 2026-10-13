use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceDocument {
    pub id: String,
    pub name: String,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnalysisRequest {
    pub revision: u32,
    pub documents: Vec<SourceDocument>,
}

/// All offsets are zero-based; lines and UTF-16 columns are one-based.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub byte_offset: u32,
    pub utf16_offset: u32,
    pub line: u32,
    pub column: u32,
}

/// Half-open range in the exact source text supplied for this snapshot.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct SourceRange {
    pub document_id: String,
    pub start: Position,
    pub end: Position,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub stage: Stage,
    pub message: String,
    pub source: SourceRange,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Syntax,
    Structure,
    Resolution,
    Semantic,
    Capability,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum RelationshipKind {
    Typing,
    Specialization,
    Subsetting,
    Redefinition,
    Import,
    Reference,
    Other,
}

/// A failed query must not masquerade as an absent relationship.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Resolution {
    Resolved {
        #[serde(rename = "targetId")]
        target_id: String,
    },
    Unresolved,
    Ambiguous,
    Unsupported {
        reason: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Reference {
    pub owner_id: Option<String>,
    pub kind: RelationshipKind,
    pub spelling: String,
    pub resolution: Resolution,
    pub source: SourceRange,
}

/// A membership is a binding, distinct from the element it names. Several
/// aliases may bind the same element without changing that element's owner.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub id: String,
    pub metaclass: String,
    pub namespace_id: String,
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub visibility: Visibility,
    pub is_alias: bool,
    pub member: Resolution,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

/// The binding's visibility in the importing namespace. Its original owner,
/// names and declared visibility remain on the referenced Membership record.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct ImportedMembership {
    pub membership_id: String,
    pub visibility: Visibility,
}

/// A checked, ordered import closure. Unknown never carries a partial set that
/// could be mistaken for a complete empty namespace.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ImportClosure {
    Complete {
        memberships: Vec<ImportedMembership>,
    },
    Unknown {
        reason: ImportIssue,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum ImportIssue {
    InvalidSyntax,
    IncompleteProvider,
    Unsupported,
    WorkLimit,
    Unavailable,
}

/// Exact decimal integer strings avoid rounding through JSON/JavaScript numbers.
/// An evaluated result proves supported bounds, not whole-model validity.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Cardinality {
    Evaluated {
        lower: String,
        upper: Option<String>,
    },
    Unknown {
        reason: CardinalityIssue,
    },
    NotApplicable,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub enum CardinalityIssue {
    InvalidSyntax,
    InvalidRelationship,
    IncompleteProvider,
    UnsupportedBound,
    CyclicDependency,
    ConflictingConstraints,
    WorkLimit,
    Unavailable,
}

/// A declaration's semantic facts. IDs are opaque and snapshot-scoped, never
/// persistent identity or array indexes. Metaclass names follow the pinned schema.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub id: String,
    pub metaclass: String,
    pub name: String,
    pub qualified_name: Option<String>,
    pub parent_id: Option<String>,
    pub library: bool,
    pub source: Option<SourceRange>,
    pub documentation: String,
    pub value_expression: Option<String>,
    pub declared_multiplicity: Option<String>,
    pub cardinality: Cardinality,
    /// Library/navigation stubs have no member-query result in this snapshot.
    pub members_available: bool,
    pub owned_members: Vec<String>,
    pub effective_members: Vec<String>,
    /// Explicit owned bindings, including aliases and anonymous memberships.
    pub owned_memberships: Vec<String>,
    /// Inherited bindings supplied by the engine; see inheritance_complete.
    pub inherited_memberships: Vec<String>,
    pub imported_memberships: ImportClosure,
    pub inheritance_complete: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub chain: Vec<String>,
    pub spelling: String,
    pub resolution: Resolution,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    pub element_id: String,
    pub ends: Vec<Endpoint>,
}

/// Owned transport value for one analysis. Treat as immutable; use `queries()`
/// to validate and borrow it. IDs require a unique caller revision per snapshot
/// in the caller's session; they are not globally unique across engines/sessions.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "bindings", derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub schema_version: u32,
    pub revision: u32,
    pub engine_version: String,
    pub library_version: String,
    pub documents: Vec<SourceDocument>,
    pub elements: Vec<Element>,
    pub memberships: Vec<Membership>,
    pub references: Vec<Reference>,
    pub connections: Vec<Connection>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Snapshot {
    pub fn element(&self, id: &str) -> Option<&Element> {
        self.elements.iter().find(|e| e.id == id)
    }
    pub fn find(&self, qualified_name: &str) -> Vec<&Element> {
        self.elements
            .iter()
            .filter(|e| e.qualified_name.as_deref() == Some(qualified_name))
            .collect()
    }
    pub fn references_at(&self, document_id: &str, byte_offset: u32) -> Vec<&Reference> {
        self.references
            .iter()
            .filter(|r| {
                r.source.document_id == document_id
                    && r.source.start.byte_offset <= byte_offset
                    && byte_offset < r.source.end.byte_offset
            })
            .collect()
    }
}

#[derive(Debug)]
pub enum AnalysisError {
    InvalidInput(String),
    Cancelled,
    Engine(String),
}

impl std::fmt::Display for AnalysisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidInput(s) | Self::Engine(s) => f.write_str(s),
            Self::Cancelled => f.write_str("Analysis cancelled"),
        }
    }
}
impl std::error::Error for AnalysisError {}
