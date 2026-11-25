use sysmlv2_model::{
    json::{ClosurePolicy, ResolvedModel},
    model::{GraphFormat, Model},
};

fn library(format: GraphFormat) -> Model {
    let mut model = Model::with_graph_format(format);
    let sources: Vec<serde_json::Value> =
        serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json"))).unwrap();
    assert_eq!(sources.len(), 94);
    for source in sources {
        model.add_library_source(
            source["name"].as_str().unwrap(),
            source["text"].as_str().unwrap(),
        );
    }
    model
}

fn exact(r: &mut ResolvedModel, name: &str, expected: &[&str]) {
    let owner = r.resolve_qualified(name).unwrap();
    let ends: Vec<_> = expected
        .iter()
        .map(|name| r.resolve_qualified(name).unwrap())
        .collect();
    let projection = r.type_feature_report(owner).projections.unwrap();
    assert_eq!(projection.end_features, ends, "{name}");
    let value = serde_json::Value::Array(
        ends.iter()
            .map(|&end| serde_json::json!({"@id":r.element_id(end)}))
            .collect(),
    );
    assert_eq!(r.property(owner, "endFeature"), Ok(value.clone()), "{name}");
    assert_eq!(r.property(owner, "associationEnd"), Ok(value), "{name}");
}

#[test]
fn association_end_queries_preserve_inheritance_and_foreign_alias_identity() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            for format in [GraphFormat::CanonicalV3, GraphFormat::LegacyV2] {
                let mut model = library(format);
                assert!(model.add_source("association-ends.kerml", r#"
                    assoc Parent specializes Links::BinaryLink {
                        end feature p:Base::Anything;
                        end feature q:Base::Anything;
                    }
                    assoc Foreign specializes Parent {
                        end feature left:Base::Anything;
                        end feature right:Base::Anything;
                        feature foreignOnly;
                    }
                    abstract assoc Aliases {alias selected for Foreign::left;}
                    assoc Merged specializes Parent,Aliases;
                    assoc Consumer specializes Merged {end feature local:Base::Anything;}
                    assoc MissingBinary {end feature p:Base::Anything; end feature q:Base::Anything;}
                "#).diagnostics.is_empty());
                let mut r = ResolvedModel::build(&model);
                let consumer = r.resolve_qualified("Consumer").unwrap();
                r.implied_relationships(consumer);
                r.set_closure_policy(ClosurePolicy::Closure { include_implied: true });
                exact(&mut r, "Consumer", &["Consumer::local"]);
                exact(&mut r, "Merged", &["Parent::q"]);
                exact(&mut r, "Aliases", &[]);
                exact(&mut r, "Links::SelfLink", &["Links::SelfLink::thisThing", "Links::SelfLink::sameThing"]);
                exact(&mut r, "Objects::BinaryLinkObject", &["Links::BinaryLink::source", "Links::BinaryLink::target"]);
                let projection = r.type_feature_report(consumer).projections.unwrap();
                let borrowed = r.resolve_qualified("Foreign::left").unwrap();
                let unrelated = r.resolve_qualified("Foreign::foreignOnly").unwrap();
                assert!(!projection.features.contains(&borrowed));
                assert!(!projection.features.contains(&unrelated));
                let members = projection.members.unwrap();
                assert!(members.contains(&borrowed));
                assert!(!members.contains(&unrelated));
                let incomplete = r.resolve_qualified("MissingBinary").unwrap();
                assert!(r.type_feature_report(incomplete).projections.is_err());
                assert!(r.property(incomplete, "associationEnd").is_err());
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
