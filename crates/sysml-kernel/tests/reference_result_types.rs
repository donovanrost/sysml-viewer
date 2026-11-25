use sysmlv2_model::{
    json::{ClosurePolicy, ResolvedModel},
    model::{GraphFormat, Model},
};

#[test]
fn generated_reference_results_expose_exact_types_through_public_queries() {
    std::thread::Builder::new()
        .stack_size(sysml_kernel::ANALYSIS_STACK_BYTES)
        .spawn(|| {
            let mut model=Model::with_graph_format(GraphFormat::CanonicalV3);
            let sources:Vec<serde_json::Value>=serde_json::from_str(include_str!(concat!(env!("OUT_DIR"),"/library.json"))).unwrap();
            assert_eq!(sources.len(),94);
            for source in sources {
                model.add_library_source(source["name"].as_str().unwrap(),source["text"].as_str().unwrap());
            }
            let parsed=model.add_source("reference-types.kerml",r#"
                datatype Small specializes ScalarValues::Integer;
                feature selectedMember:Small;
                feature queried=selectedMember;
                function F {in x:ScalarValues::Integer default=1;return result:ScalarValues::Integer;}
                feature call=F();
            "#);
            assert!(parsed.diagnostics.is_empty(),"{:?}",parsed.diagnostics);
            let mut graph=ResolvedModel::build(&model);
            let reference=graph.user_elements().find(|&e|graph.element_type(e)=="FeatureReferenceExpression").unwrap();
            let call=graph.user_elements().find(|&e|graph.element_type(e)=="InvocationExpression").unwrap();
            graph.implied_relationships(call);
            graph.set_closure_policy(ClosurePolicy::Closure{include_implied:true});
            let result=graph.expression_result_report(reference).result.unwrap().unwrap();
            let small=graph.resolve_qualified("Small").unwrap();
            assert_eq!(graph.owner(result),Some(reference));
            assert_eq!(graph.feature_type_report(result).types,Ok(vec![small]));
            assert_eq!(graph.property(result,"type").unwrap(),serde_json::json!([{"@id":graph.element_id(small)}]));
            let defaults=graph.invocation_default_report(call).selection.unwrap().defaults;
            assert_eq!(defaults.len(),10);
            let mut checked=0;
            for default in defaults {
                let name=graph.element_qualified_name(default.feature).unwrap();
                let expected=match name.as_str(){
                    "F::x"=>"ScalarValues::Integer",
                    "Performances::Performance::isDispatch" | "Occurrences::Occurrence::isRunToCompletion"=>"ScalarValues::Boolean",
                    "Performances::Performance::dispatchScope"=>"Performances::Performance",
                    "Occurrences::Occurrence::localClock"=>"Clocks::UniversalClockLife",
                    "Occurrences::Occurrence::incomingTransferSort"=>"Occurrences::IncomingTransferSort",
                    _=>continue,
                };
                let result=graph.expression_result_report(default.value).result.unwrap().unwrap();
                let expected=graph.resolve_qualified(expected).unwrap();
                assert_eq!(graph.feature_type_report(result).types,Ok(vec![expected]),"{name}");
                assert_eq!(graph.property(result,"type").unwrap(),serde_json::json!([{"@id":graph.element_id(expected)}]),"{name}");
                checked+=1;
            }
            assert_eq!(checked,6);
        }).unwrap().join().unwrap();
}
