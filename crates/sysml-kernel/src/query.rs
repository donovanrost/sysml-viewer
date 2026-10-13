use crate::{
    Cardinality, Diagnostic, Element, Membership, Reference, Resolution, Snapshot, SourceDocument,
    SourceIndex, SourceRange, SCHEMA_VERSION,
};
use std::collections::HashMap;

/// Query failures are distinct from successful empty collections.
#[derive(Debug, PartialEq, Eq)]
pub enum QueryError {
    InvalidSnapshot(String),
    UnknownElement(String),
    UnknownMembership(String),
    UnknownDocument(String),
    MembersUnavailable(String),
    ReferencesUnavailable(String),
    DiagnosticsUnavailable(String),
    InvalidSourceRange,
}
impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for QueryError {}

/// All projected members, including library members. Completeness describes
/// inherited-member computation, not full language validation or metamodel coverage.
#[derive(Debug)]
pub struct EffectiveMembers<'a> {
    pub members: Vec<&'a Element>,
    pub inheritance_complete: bool,
}

#[derive(Debug)]
pub struct InheritedMemberships<'a> {
    pub memberships: Vec<&'a Membership>,
    pub inheritance_complete: bool,
}

/// An indexed immutable borrow of a structurally validated snapshot. Construction
/// validates handle/range integrity, not the modeled system's semantic validity.
pub struct SnapshotQuery<'a> {
    snapshot: &'a Snapshot,
    elements: HashMap<&'a str, &'a Element>,
    memberships: HashMap<&'a str, &'a Membership>,
    documents: HashMap<&'a str, &'a SourceDocument>,
    names: HashMap<&'a str, Vec<&'a Element>>,
}

impl Snapshot {
    pub fn queries(&self) -> Result<SnapshotQuery<'_>, QueryError> {
        SnapshotQuery::new(self)
    }
}

impl<'a> SnapshotQuery<'a> {
    fn new(snapshot: &'a Snapshot) -> Result<Self, QueryError> {
        let invalid = |reason: &str| QueryError::InvalidSnapshot(reason.into());
        if snapshot.schema_version != SCHEMA_VERSION {
            return Err(invalid("Unsupported schema version"));
        }
        let mut query = Self {
            snapshot,
            elements: HashMap::new(),
            memberships: HashMap::new(),
            documents: HashMap::new(),
            names: HashMap::new(),
        };
        for doc in &snapshot.documents {
            if doc.id.is_empty() || query.documents.insert(&doc.id, doc).is_some() {
                return Err(invalid("Empty or duplicate document identity"));
            }
        }
        for element in &snapshot.elements {
            if element.id.is_empty() || query.elements.insert(&element.id, element).is_some() {
                return Err(invalid("Empty or duplicate element identity"));
            }
            if let Some(name) = &element.qualified_name {
                query.names.entry(name).or_default().push(element);
            }
        }
        for membership in &snapshot.memberships {
            if membership.id.is_empty()
                || query
                    .memberships
                    .insert(&membership.id, membership)
                    .is_some()
            {
                return Err(invalid("Empty or duplicate membership identity"));
            }
        }
        let indexes: HashMap<_, _> = snapshot
            .documents
            .iter()
            .map(|doc| (doc.id.as_str(), SourceIndex::new(&doc.id, &doc.text)))
            .collect();
        let validate_range = |range: &SourceRange| -> Result<(), QueryError> {
            query.source(range)?;
            let index = &indexes[range.document_id.as_str()];
            if index.position(range.start.byte_offset) != range.start
                || index.position(range.end.byte_offset) != range.end
            {
                return Err(QueryError::InvalidSourceRange);
            }
            Ok(())
        };
        let validate_resolution = |resolution: &Resolution| -> Result<(), QueryError> {
            if let Resolution::Resolved { target_id } = resolution {
                query.element(target_id)?;
            }
            Ok(())
        };
        for membership in &snapshot.memberships {
            query.element(&membership.namespace_id)?;
            validate_resolution(&membership.member)?;
        }
        for element in &snapshot.elements {
            for id in element
                .parent_id
                .iter()
                .chain(&element.owned_members)
                .chain(&element.effective_members)
            {
                query.element(id)?;
            }
            if !element.members_available
                && (!element.owned_members.is_empty()
                    || !element.effective_members.is_empty()
                    || !element.owned_memberships.is_empty()
                    || !element.inherited_memberships.is_empty())
            {
                return Err(invalid("Unavailable member query contains results"));
            }
            for id in &element.owned_memberships {
                if query.membership(id)?.namespace_id != element.id {
                    return Err(invalid("Owned membership belongs to another namespace"));
                }
            }
            for id in &element.inherited_memberships {
                query.membership(id)?;
            }
            if let Cardinality::Evaluated { lower, upper } = &element.cardinality {
                let number = |s: &str| -> Result<i128, QueryError> {
                    let n = s
                        .parse::<i128>()
                        .map_err(|_| invalid("Invalid cardinality integer"))?;
                    if n < 0 || n.to_string() != s {
                        return Err(invalid("Noncanonical cardinality integer"));
                    }
                    Ok(n)
                };
                let low = number(lower)?;
                if let Some(high) = upper {
                    if number(high)? < low {
                        return Err(invalid("Reversed cardinality interval"));
                    }
                }
            }
            if let Some(range) = &element.source {
                validate_range(range)?;
            }
        }
        for reference in &snapshot.references {
            if let Some(id) = &reference.owner_id {
                query.element(id)?;
            }
            validate_resolution(&reference.resolution)?;
            validate_range(&reference.source)?;
        }
        for connection in &snapshot.connections {
            query.element(&connection.element_id)?;
            for end in &connection.ends {
                for id in &end.chain {
                    query.element(id)?;
                }
                validate_resolution(&end.resolution)?;
                if let Resolution::Resolved { target_id } = &end.resolution {
                    if end.chain.last() != Some(target_id) {
                        return Err(invalid(
                            "Resolved endpoint does not terminate at its target",
                        ));
                    }
                }
            }
        }
        for diagnostic in &snapshot.diagnostics {
            validate_range(&diagnostic.source)?;
        }
        Ok(query)
    }

    pub fn element(&self, id: &str) -> Result<&'a Element, QueryError> {
        self.elements
            .get(id)
            .copied()
            .ok_or_else(|| QueryError::UnknownElement(id.into()))
    }

    /// Canonical-name lookup only. This does not resolve a textual name in scope
    /// or enumerate aliases. Zero/multiple matches are preserved, never guessed.
    pub fn find(&self, qualified_name: &str) -> &[&'a Element] {
        self.names
            .get(qualified_name)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    pub fn document(&self, id: &str) -> Result<&'a SourceDocument, QueryError> {
        self.documents
            .get(id)
            .copied()
            .ok_or_else(|| QueryError::UnknownDocument(id.into()))
    }

    pub fn membership(&self, id: &str) -> Result<&'a Membership, QueryError> {
        self.memberships
            .get(id)
            .copied()
            .ok_or_else(|| QueryError::UnknownMembership(id.into()))
    }

    pub fn owned_memberships(&self, id: &str) -> Result<Vec<&'a Membership>, QueryError> {
        self.member_owner(id)?
            .owned_memberships
            .iter()
            .map(|id| self.membership(id))
            .collect()
    }

    /// Retains membership identity, aliases and completeness. This is not a
    /// complete namespace-membership closure (imports are not enumerated).
    pub fn inherited_memberships(&self, id: &str) -> Result<InheritedMemberships<'a>, QueryError> {
        let element = self.member_owner(id)?;
        Ok(InheritedMemberships {
            memberships: element
                .inherited_memberships
                .iter()
                .map(|id| self.membership(id))
                .collect::<Result<_, _>>()?,
            inheritance_complete: element.inheritance_complete,
        })
    }

    pub fn cardinality(&self, id: &str) -> Result<&'a Cardinality, QueryError> {
        Ok(&self.element(id)?.cardinality)
    }

    pub fn owned_members(&self, id: &str) -> Result<Vec<&'a Element>, QueryError> {
        let element = self.member_owner(id)?;
        element
            .owned_members
            .iter()
            .map(|id| self.element(id))
            .collect()
    }

    pub fn effective_members(&self, id: &str) -> Result<EffectiveMembers<'a>, QueryError> {
        let element = self.member_owner(id)?;
        Ok(EffectiveMembers {
            members: element
                .effective_members
                .iter()
                .map(|id| self.element(id))
                .collect::<Result<_, _>>()?,
            inheritance_complete: element.inheritance_complete,
        })
    }

    fn member_owner(&self, id: &str) -> Result<&'a Element, QueryError> {
        let element = self.element(id)?;
        if !element.members_available {
            return Err(QueryError::MembersUnavailable(id.into()));
        }
        Ok(element)
    }

    pub fn references_from(&self, id: &str) -> Result<Vec<&'a Reference>, QueryError> {
        if self.element(id)?.library {
            return Err(QueryError::ReferencesUnavailable(id.into()));
        }
        Ok(self
            .snapshot
            .references
            .iter()
            .filter(|r| r.owner_id.as_deref() == Some(id))
            .collect())
    }

    pub fn references_at(
        &self,
        document_id: &str,
        byte_offset: u32,
    ) -> Result<Vec<&'a Reference>, QueryError> {
        let doc = self.document(document_id)?;
        if document_id.starts_with("library://") {
            return Err(QueryError::ReferencesUnavailable(document_id.into()));
        }
        if !doc.text.is_char_boundary(byte_offset as usize) {
            return Err(QueryError::InvalidSourceRange);
        }
        Ok(self.snapshot.references_at(document_id, byte_offset))
    }

    pub fn diagnostics(&self, document_id: &str) -> Result<Vec<&'a Diagnostic>, QueryError> {
        self.document(document_id)?;
        if document_id.starts_with("library://") {
            return Err(QueryError::DiagnosticsUnavailable(document_id.into()));
        }
        Ok(self
            .snapshot
            .diagnostics
            .iter()
            .filter(|d| d.source.document_id == document_id)
            .collect())
    }

    /// Slice exact source using UTF-8 byte offsets. The range must belong to this
    /// snapshot; do not pass handles or ranges saved from another request.
    pub fn source(&self, range: &SourceRange) -> Result<&'a str, QueryError> {
        self.document(&range.document_id)?
            .text
            .get(range.start.byte_offset as usize..range.end.byte_offset as usize)
            .ok_or(QueryError::InvalidSourceRange)
    }
}
