use sysmlv2_model::{check, eval::Value, json::ResolvedModel, model::Model, rational::Rational};

fn model(source: &str) -> Model {
    let mut model = Model::new();
    // Minimal loaded standard-library declarations retain their normative
    // KerML 9.1 identities. User declarations with these names do not.
    let parsed = model.add_library_source(
        "scalars.kerml",
        "standard library package ScalarValues {
            datatype Boolean; datatype String; datatype Real;
            datatype Integer specializes Real;
        }",
    );
    assert!(parsed.diagnostics.is_empty());
    let parsed = model.add_source("test.kerml", source);
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    model
}

#[test]
fn scalar_partitions_follow_library_identity_not_simple_names() {
    let model = model(
        r#"package P {
            datatype Integer specializes ScalarValues::String;
            datatype Boolean specializes ScalarValues::Integer;
            datatype String specializes ScalarValues::Boolean;
            feature text : Integer = "hello";
            feature count : Boolean = 2;
            feature flag : String = true;
        }"#,
    );
    let mut graph = ResolvedModel::build(&model);
    let diagnostics = check::validate_semantics_with(&mut graph, &model);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for (name, value) in [
        ("P::Integer", Value::String("hello".into())),
        ("P::Boolean", Value::Integer(2)),
        ("P::String", Value::Boolean(true)),
    ] {
        let ty = graph.resolve_qualified(name).unwrap();
        assert_eq!(
            graph.scalar_value_admitted(&[ty], &value),
            Some(true),
            "{name}"
        );
    }
}

#[test]
fn every_declared_type_and_every_supertype_restricts_the_value() {
    // KerML 8.3.3.3 Feature::type requires membership in every type;
    // 7.3.2.2 specialization carries all general-type restrictions.
    let model = model(
        "package P {
            datatype CountA specializes ScalarValues::Real, ScalarValues::Integer;
            datatype CountB specializes ScalarValues::Integer, ScalarValues::Real;
            datatype Unclassified;
            feature a : ScalarValues::Real, ScalarValues::Integer = 1.5;
            feature b : ScalarValues::Integer, ScalarValues::Real = 1.5;
            feature c : CountA = 1.5;
            feature d : CountB = 1.5;
            feature e : ScalarValues::String, ScalarValues::Integer = 2;
            feature f : Unclassified, ScalarValues::Integer = 1.5;
        }",
    );
    let mut graph = ResolvedModel::build(&model);
    let diagnostics = check::validate_semantics_with(&mut graph, &model);
    assert_eq!(diagnostics.len(), 6, "{diagnostics:?}");
    for names in [
        vec!["ScalarValues::Real", "ScalarValues::Integer"],
        vec!["ScalarValues::Integer", "ScalarValues::Real"],
        vec!["P::CountA"],
        vec!["P::CountB"],
        vec!["P::Unclassified", "ScalarValues::Integer"],
    ] {
        let types: Vec<_> = names
            .iter()
            .map(|n| graph.resolve_qualified(n).unwrap())
            .collect();
        assert_eq!(
            graph.scalar_value_admitted(
                &types,
                &Value::Rational(Rational::parse_decimal("1.5").unwrap())
            ),
            Some(false),
            "{names:?}"
        );
        let expected = if names.contains(&"P::Unclassified") {
            None
        } else {
            Some(true)
        };
        assert_eq!(
            graph.scalar_value_admitted(&types, &Value::Integer(2)),
            expected,
            "{names:?}"
        );
    }
}

#[test]
fn unclassified_and_unloaded_names_do_not_prove_scalar_compatibility() {
    let mut model = Model::new();
    assert!(model
        .add_source(
            "unloaded.kerml",
            "package ScalarValues { datatype Integer; }"
        )
        .diagnostics
        .is_empty());
    let mut graph = ResolvedModel::build(&model);
    let ty = graph.resolve_qualified("ScalarValues::Integer").unwrap();
    assert_eq!(
        graph.scalar_value_admitted(&[ty], &Value::String("text".into())),
        None
    );
    assert_eq!(graph.scalar_value_admitted(&[], &Value::Integer(1)), None);
}

#[test]
fn restrictions_survive_transitive_subsetting_and_redefinition() {
    let model = model(
        "package P {
            class Parent {feature count : ScalarValues::Integer;}
            class Child specializes Parent {feature count redefines Parent::count;}
            class Grandchild specializes Child {feature count redefines Child::count = 1.5;}
            feature base : ScalarValues::Integer;
            feature intermediate subsets base;
            feature narrowed : ScalarValues::Real subsets intermediate = 1.5;
        }",
    );
    let mut graph = ResolvedModel::build(&model);
    let diagnostics = check::validate_semantics_with(&mut graph, &model);
    let scalar_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|(_, d)| d.message.contains("not an integer"))
        .collect();
    assert_eq!(scalar_diagnostics.len(), 2, "{diagnostics:?}");
}

#[test]
fn exhausted_walks_and_unclassified_cycles_cannot_prove_compatibility() {
    let mut source = String::from("datatype T0 specializes ScalarValues::Integer;");
    for i in 1..70 {
        source.push_str(&format!("datatype T{i} specializes T{};", i - 1));
    }
    source.push_str("datatype A specializes B; datatype B specializes A;");
    let model = model(&source);
    let mut graph = ResolvedModel::build(&model);
    for name in ["T69", "A"] {
        let ty = graph.resolve_qualified(name).unwrap();
        assert_eq!(
            graph.scalar_value_admitted(&[ty], &Value::Integer(2)),
            None,
            "{name}"
        );
    }
}
