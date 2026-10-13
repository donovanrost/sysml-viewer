//! Reproducible API probe; this is deliberately separate from product dependencies.
use serde_json::json;
use std::{fs, path::{Path, PathBuf}};
use sysmlv2_model::{json::ResolvedModel, model::Model};

fn sources(path: &Path) -> Vec<(String, String)> {
    fn walk(path: &Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(path).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() { walk(&p, out); }
            else if matches!(p.extension().and_then(|e| e.to_str()), Some("sysml" | "kerml")) { out.push(p); }
        }
    }
    let mut files = Vec::new(); walk(path, &mut files); files.sort();
    files.into_iter().map(|p| (p.strip_prefix(path).unwrap().to_string_lossy().replace('\\', "/"), fs::read_to_string(p).unwrap())).collect()
}

fn main() {
    std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(run).unwrap().join().unwrap();
}

fn run() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3, "usage: kernel-evaluation FIXTURES LIBRARY");
    let docs = sources(Path::new(&args[1]));
    let library = sources(Path::new(&args[2]));
    let mut model = Model::new();
    for (name, text) in &library { model.add_library_source(name, text); }
    for (name, text) in &docs { model.add_source(name, text); }
    let mut r = ResolvedModel::build(&model);
    let a = r.resolve_qualified("Mission::vehicle::a").unwrap();
    let ty = r.typings(a).into_iter().map(|e| r.element_qualified_name(e)).collect::<Vec<_>>();
    let members = r.effective_features(a, false).into_iter().map(|e| r.element_qualified_name(e)).collect::<Vec<_>>();
    let connectors = r.elements_of_metaclass("ConnectionUsage");
    let endpoints = connectors.into_iter().filter(|e| !r.is_library_element(*e)).collect::<Vec<_>>().into_iter().map(|e| {
        r.connector_end_targets(e).into_iter().map(|end| end.chain.into_iter().map(|link| r.element_qualified_name(link)).collect::<Vec<_>>()).collect::<Vec<_>>()
    }).collect::<Vec<_>>();
    let report = sysmlv2_model::json::model_resolution_report(&model);
    let toolkit = json!({
        "types": ty, "effectiveMembers": members, "endpoints": endpoints,
        "ambiguous": report.ambiguous.iter().filter(|(u,_)| !model.is_library_unit(*u)).map(|(_,q)| q.to_display_string()).collect::<Vec<_>>(),
        "unresolved": report.unresolved.iter().filter(|(u,_)| !model.is_library_unit(*u)).map(|(_,q)| q.to_display_string()).collect::<Vec<_>>(),
        "parseErrors": model.units().iter().filter(|u| !u.is_library).map(|u| u.diagnostics.len()).sum::<usize>(),
    });

    let mut host = syster::ide::AnalysisHost::new();
    for (name, text) in &library { host.set_file_content(&format!("library/{name}"), text); }
    let parse_errors = docs.iter().map(|(name, text)| host.set_file_content(name, text).len()).sum::<usize>();
    let analysis = host.analysis();
    let index = analysis.symbol_index();
    let name = |q: &str| index.lookup_qualified(q).map(|s| s.qualified_name.to_string());
    let lookup = |scope: &str, n: &str| index.resolver_for_scope(scope).resolve(n).symbol().map(|s| s.qualified_name.to_string());
    let members = ["signal", "mass", "serial"].map(|n| index.find_member_in_scope("Mission::vehicle::a", n).map(|s| s.qualified_name.to_string()));
    let syster = json!({
        "usage": name("Mission::vehicle::a"),
        "importedType": lookup("Mission::vehicle", "SpecialDevice"),
        "membersByName": members,
        "standardType": lookup("Components::Device", "Real"),
        "ambiguous": index.resolver_for_scope("Problems").resolve("Duplicate").is_ambiguous(),
        "parseErrors": parse_errors,
    });
    println!("{}", serde_json::to_string_pretty(&json!({"openMbee":toolkit, "syster":syster})).unwrap());
}
