//! Direct dependency queries must not rely on the facade's earlier model scan.
use sysml_kernel::ANALYSIS_STACK_BYTES;
use sysmlv2_model::{
    json::{CardinalityBounds, ResolvedModel},
    model::{GraphFormat, Model},
};

#[test]
fn first_compound_range_query_fits_the_budget_with_the_full_library() {
    std::thread::Builder::new()
        .stack_size(ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let sources: Vec<serde_json::Value> =
                serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/library.json")))
                    .unwrap();
            assert_eq!(sources.len(), 94);
            for with_missing_base in [false, true] {
                // Each endpoint form is the FIRST cardinality query on a new
                // model. Warm retries cannot make this regression pass.
                for (first, lower, upper) in
                    [("range", 5, 10), ("viaAlias", 5, 10), ("repeated", 10, 10)]
                {
                    let mut model = Model::with_graph_format(GraphFormat::CanonicalV3);
                    for source in &sources {
                        model.add_library_source(
                            source["name"].as_str().unwrap(),
                            source["text"].as_str().unwrap(),
                        );
                    }
                    model.add_source(
                        "ranges.sysml",
                        include_str!("../../../tests/conformance/fixtures/mult-022.sysml"),
                    );
                    if with_missing_base {
                        model.add_source(
                            "missing.sysml",
                            "package BrokenRangeContext { part def Base :> MissingBase; }",
                        );
                    }
                    let mut resolved = ResolvedModel::build(&model);
                    let element = resolved
                        .resolve_qualified(&format!("CompoundRanges::{first}"))
                        .unwrap();
                    // Query the supported canonical graph with implied
                    // relationships published, but no preceding proof query.
                    resolved.implied_relationships(element);
                    let cold = resolved.cardinality_report(element);
                    assert_eq!(
                        cold.bounds,
                        Ok(CardinalityBounds {
                            lower,
                            upper: Some(upper),
                        }),
                        "{first}, missing base={with_missing_base}: {cold:?}",
                    );
                    let warm = resolved.cardinality_report(element);
                    assert_eq!(warm.bounds, cold.bounds);
                    assert_eq!(warm.multiplicities, cold.multiplicities);
                    assert!(warm.steps < cold.steps);
                }
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
