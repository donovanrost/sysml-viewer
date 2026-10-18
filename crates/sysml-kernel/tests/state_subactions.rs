use std::sync::OnceLock;
use sysml_kernel::*;

fn snapshot() -> &'static Snapshot {
    static VALUE: OnceLock<Snapshot> = OnceLock::new();
    VALUE.get_or_init(|| std::thread::Builder::new().stack_size(ANALYSIS_STACK_BYTES).spawn(|| {
        Engine::bundled().analyze(AnalysisRequest { revision: 52, documents: vec![
            SourceDocument { id: "bindings".into(), name: "bindings.sysml".into(), text: include_str!("../../../tests/conformance/fixtures/state-query-bindings.sysml").into() },
            SourceDocument { id: "recovery".into(), name: "recovery.sysml".into(), text: include_str!("../../../tests/conformance/fixtures/state-query-recovery.sysml").into() },
        ] }, || false).unwrap()
    }).unwrap().join().unwrap())
}

fn actions<'a>(query: &'a SnapshotQuery<'a>, name: &str) -> &'a [StateSubaction] {
    let StateSubactions::Complete { actions } =
        query.state_subactions(&query.find(name)[0].id).unwrap()
    else {
        panic!("Expected complete bindings for {name}")
    };
    actions
}

#[test]
fn ordered_bindings_preserve_roles_ownership_and_exact_action_identity() {
    let query = snapshot().queries().unwrap();
    let records = actions(&query, "StateQueries::Ordered");
    assert_eq!(
        records.iter().map(|a| a.kind).collect::<Vec<_>>(),
        [
            StateSubactionKind::Exit,
            StateSubactionKind::Entry,
            StateSubactionKind::Do
        ]
    );
    for (binding, name) in records.iter().zip(["leaving", "entering", "running"]) {
        let action = query.element(&binding.action_id).unwrap();
        let member = query.membership(&binding.membership_id).unwrap();
        assert_eq!(action.name, name);
        assert_eq!(member.metaclass, "StateSubactionMembership");
        assert_eq!(
            member.member,
            Resolution::Resolved {
                target_id: action.id.clone()
            }
        );
        assert_eq!(action.parent_id.as_ref(), Some(&member.namespace_id));
    }
    let performed = actions(&query, "StateQueries::performed");
    assert_eq!(performed.len(), 1);
    assert_eq!(
        query
            .element(&performed[0].action_id)
            .unwrap()
            .qualified_name
            .as_deref(),
        Some("StateQueries::performed::invoked")
    );
    assert_ne!(
        performed[0].action_id,
        query.find("StateQueries::external")[0].id
    );
}

#[test]
fn inherited_members_do_not_become_owned_subactions_and_duplicates_remain_visible() {
    let query = snapshot().queries().unwrap();
    assert!(actions(&query, "StateQueries::Inherited").is_empty());
    assert!(actions(&query, "StateQueries::inherited").is_empty());
    assert_eq!(actions(&query, "StateQueries::local").len(), 1);
    let duplicate = actions(&query, "StateQueries::Duplicates");
    assert_eq!(duplicate.len(), 2);
    assert!(duplicate
        .iter()
        .all(|a| a.kind == StateSubactionKind::Entry));
    assert_ne!(duplicate[0].membership_id, duplicate[1].membership_id);
    assert!(snapshot()
        .diagnostics
        .iter()
        .any(|d| d.code == "validateStateDefinitionStateSubactionKind"));
    assert_eq!(actions(&query, "StateQueries::Anonymous").len(), 3);
}

#[test]
fn syntax_recovery_stubs_and_missing_action_types_have_distinct_results() {
    let query = snapshot().queries().unwrap();
    assert_eq!(
        query
            .state_subactions(&query.find("StateQueryRecovery::Recovered")[0].id)
            .unwrap(),
        &StateSubactions::Unknown {
            reason: StateSubactionIssue::InvalidSyntax
        }
    );
    assert_eq!(
        actions(&query, "StateQueries::missing").len(),
        1,
        "The owned action exists even if its type is unavailable"
    );
    assert_eq!(
        query
            .state_subactions(&query.find("States::StateAction")[0].id)
            .unwrap(),
        &StateSubactions::Unknown {
            reason: StateSubactionIssue::Unavailable
        }
    );
    assert_eq!(
        query
            .state_subactions(&query.find("StateQueries::NotState")[0].id)
            .unwrap(),
        &StateSubactions::NotApplicable
    );
    assert!(matches!(
        query.state_subactions("stale-handle"),
        Err(QueryError::UnknownElement(_))
    ));
}

#[test]
fn deserialized_binding_queries_reject_missing_reordered_or_rebound_handles() {
    let baseline: Snapshot =
        serde_json::from_str(&serde_json::to_string(snapshot()).unwrap()).unwrap();
    baseline.queries().unwrap();
    for mutation in 0..5 {
        let mut value = baseline.clone();
        let owner = value
            .elements
            .iter_mut()
            .find(|e| e.qualified_name.as_deref() == Some("StateQueries::Ordered"))
            .unwrap();
        let StateSubactions::Complete { actions } = &mut owner.state_subactions else {
            panic!()
        };
        match mutation {
            0 => {
                actions.pop();
            }
            1 => actions.swap(0, 1),
            2 => actions[0].action_id = actions[1].action_id.clone(),
            3 => actions[0].membership_id = "absent-binding".into(),
            _ => {
                actions.push(actions[0].clone());
            }
        }
        assert!(value.queries().is_err(), "mutation {mutation}");
    }
}

#[test]
fn old_wire_schema_and_unsupported_applicability_are_rejected() {
    let mut value = snapshot().clone();
    value.schema_version = 3;
    assert!(value.queries().is_err());
    value.schema_version = SCHEMA_VERSION;
    let state = value
        .elements
        .iter_mut()
        .find(|e| e.qualified_name.as_deref() == Some("StateQueries::Ordered"))
        .unwrap();
    state.state_subactions = StateSubactions::NotApplicable;
    assert!(value.queries().is_err());
}
