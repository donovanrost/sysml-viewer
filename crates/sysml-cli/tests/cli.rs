use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "sysml-cli-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str, text: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn run(&self, args: &[&str], input: Option<&str>) -> Output {
        self.invoke(env!("CARGO_BIN_EXE_sysml"), args, input)
    }
    fn invoke(&self, binary: &str, args: &[&str], input: Option<&str>) -> Output {
        let mut child = Command::new(binary)
            .current_dir(&self.0)
            .args(args)
            .stdin(if input.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Some(input) = input {
            child
                .stdin
                .take()
                .unwrap()
                .write_all(input.as_bytes())
                .unwrap();
        }
        child.wait_with_output().unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn status(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
fn report(output: &Output) -> Value {
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn help_version_and_bad_invocations_do_not_analyze() {
    let p = Project::new();
    for args in [
        vec![],
        vec!["--help"],
        vec!["help", "validate"],
        vec!["inspect", "--help"],
        vec!["--version"],
    ] {
        let output = p.run(&args, None);
        status(&output, 0);
        assert!(!output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
    for args in [
        vec!["unknown"],
        vec!["validate"],
        vec!["parse", "--typo"],
        vec!["parse", "--format"],
        vec!["parse", "--format=yaml", "-"],
        vec!["inspect", "--format=text", "-"],
        vec!["validate", "--request-json", "a.sysml"],
        vec!["parse", "-", "-"],
        vec!["parse", "--stdin-name", "a.kerml", "a.sysml"],
    ] {
        let output = p.run(&args, None);
        status(&output, 2);
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn parse_checks_syntax_without_resolving_names_and_preserves_json_locations() {
    let p = Project::new();
    let output = p.run(
        &["parse", "--format", "json", "-"],
        Some("package P { part p : UnknownType; }"),
    );
    status(&output, 0);
    let value = report(&output);
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["command"], "parse");
    assert_eq!(value["passed"], true);
    assert_eq!(value["libraryVersion"], Value::Null);
    assert_eq!(value["summary"]["files"], 1);
    let broken = p.run(
        &["parse", "--format=json", "-"],
        Some("package P {\r\n  part p : ;\r\n}"),
    );
    status(&broken, 1);
    let value = report(&broken);
    assert_eq!(value["passed"], false);
    let diagnostic = &value["diagnostics"][0];
    assert_eq!(diagnostic["source"]["documentId"], "stdin://model");
    assert_eq!(diagnostic["source"]["start"]["line"], 2);
    assert!(diagnostic["source"]["start"]["column"].as_u64().unwrap() > 1);
    assert_eq!(diagnostic["stage"], "syntax");
}

#[test]
fn text_diagnostics_use_stderr_and_source_names() {
    let p = Project::new();
    let output = p.run(
        &["parse", "--stdin-name", "draft.sysml", "-"],
        Some("package P { part p : ; }"),
    );
    status(&output, 1);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("draft.sysml:1:"));
    assert!(stderr.contains("error["));
    assert!(String::from_utf8_lossy(&output.stdout).starts_with("Syntax checked 1 file(s):"));
}

#[test]
fn directory_discovery_is_recursive_sorted_deduplicated_and_dialect_aware() {
    let p = Project::new();
    p.file("models/a.sysml", "package A { part def Thing; }");
    p.file("models/sub/b.KERML", "package B { class Thing; }");
    p.file("models/.ignored/bad.sysml", "nonsense @@@");
    p.file("models/readme.txt", "not a model");
    p.file("-draft.sysml", "package C;");
    let output = p.run(
        &[
            "parse",
            "--format=json",
            "models",
            "models/a.sysml",
            "--",
            "-draft.sysml",
        ],
        None,
    );
    status(&output, 0);
    let value = report(&output);
    assert_eq!(value["summary"]["files"], 3);
    assert_eq!(
        value["documents"][0]["name"]
            .as_str()
            .unwrap()
            .replace('\\', "/"),
        "models/a.sysml"
    );
    assert_eq!(
        value["documents"][1]["name"]
            .as_str()
            .unwrap()
            .replace('\\', "/"),
        "models/sub/b.KERML"
    );
    let kerml = p.run(
        &["parse", "--stdin-name", "stdin.kerml", "-"],
        Some("package P { class Thing; }"),
    );
    status(&kerml, 0);
}

#[test]
fn invalid_inputs_fail_with_exit_two_and_no_partial_json() {
    let p = Project::new();
    p.file("bad.txt", "package P;");
    fs::create_dir(p.0.join("empty")).unwrap();
    fs::write(p.0.join("invalid.sysml"), [0xff, 0xfe]).unwrap();
    for path in ["missing.sysml", "empty", "bad.txt", "invalid.sysml"] {
        let output = p.run(&["parse", "--format=json", path], None);
        status(&output, 2);
        assert!(output.stdout.is_empty());
    }
    let output = p.run(&["parse", "--request-json"], Some("not json"));
    status(&output, 2);
    assert!(output.stdout.is_empty());
    for i in 0..33 {
        p.file(&format!("many/{i:02}.sysml"), "package P;");
    }
    status(&p.run(&["parse", "many"], None), 2);
    p.file("large.sysml", &" ".repeat(5_000_001));
    status(&p.run(&["parse", "large.sysml"], None), 2);
}

#[test]
fn json_request_preserves_revision_and_enforces_shared_workspace_policy() {
    let p = Project::new();
    let doc = json!({"id":"one", "name":"one.sysml", "text":"package P;"});
    let request = json!({"revision":42, "documents":[doc.clone()]});
    let output = p.run(
        &["parse", "--request-json", "--format=json"],
        Some(&request.to_string()),
    );
    status(&output, 0);
    assert_eq!(report(&output)["revision"], 42);
    let duplicate = json!({"revision":1, "documents":[doc.clone(), doc]});
    let output = p.run(
        &["parse", "--request-json", "--format=json"],
        Some(&duplicate.to_string()),
    );
    status(&output, 2);
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unique"));
}

#[test]
fn validation_resolves_the_whole_directory_and_inspection_keeps_snapshot_contract() {
    let p = Project::new();
    p.file(
        "models/Definitions.sysml",
        "package Types { part def Vessel; }",
    );
    p.file(
        "models/Mission.sysml",
        "package Mission { part vessel : Types::Vessel; }",
    );
    let output = p.run(&["validate", "--format=json", "models"], None);
    status(&output, 0);
    let value = report(&output);
    assert_eq!(value["command"], "validate");
    assert_eq!(value["summary"]["errors"], 0);
    assert_eq!(value["summary"]["files"], 2);
    assert!(value["libraryVersion"].is_string());
    let output = p.run(&["inspect", "models", "--strict"], None);
    status(&output, 0);
    let value = report(&output);
    assert_eq!(value["schemaVersion"], 4);
    assert!(value["references"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["spelling"] == "Types::Vessel" && r["resolution"]["status"] == "resolved"));
    assert!(value["elements"].is_array());
}

#[test]
fn error_diagnostics_fail_validation_but_inspect_and_legacy_remain_opt_in() {
    let p = Project::new();
    p.file("broken.sysml", "package P { part p : ; }");
    let output = p.run(&["validate", "--format=json", "broken.sysml"], None);
    status(&output, 1);
    assert_eq!(report(&output)["passed"], false);
    for strict in [false, true] {
        let mut args = vec!["inspect", "broken.sysml"];
        if strict {
            args.push("--strict");
        }
        let output = p.run(&args, None);
        status(&output, i32::from(strict));
        assert_eq!(report(&output)["schemaVersion"], 4);
        let output = p.invoke(env!("CARGO_BIN_EXE_sysml-inspect"), &args[1..], None);
        status(&output, i32::from(strict));
        assert_eq!(report(&output)["schemaVersion"], 4);
    }
    let request =
        json!({"revision":9, "documents":[{"id":"legacy", "name":"x.sysml", "text":"package P;"}]});
    let output = p.invoke(
        env!("CARGO_BIN_EXE_sysml-inspect"),
        &["--request-json"],
        Some(&request.to_string()),
    );
    status(&output, 0);
    assert_eq!(report(&output)["revision"], 9);
}

#[test]
fn warnings_keep_kernel_severity_and_can_fail_ci_explicitly() {
    let p = Project::new();
    p.file("warning.sysml", "package P { part p : MissingType; }");
    let output = p.run(&["validate", "--format=json", "warning.sysml"], None);
    status(&output, 0);
    let value = report(&output);
    assert_eq!(value["summary"]["errors"], 0);
    assert!(value["summary"]["warnings"].as_u64().unwrap() > 0);
    let output = p.run(
        &[
            "validate",
            "--format=json",
            "--warnings-as-errors",
            "warning.sysml",
        ],
        None,
    );
    status(&output, 1);
    let value = report(&output);
    assert_eq!(value["passed"], false);
    assert!(value["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .all(|d| d["severity"] == "warning"));
    let output = p.run(&["inspect", "--warnings-as-errors", "warning.sysml"], None);
    status(&output, 1);
    assert_eq!(report(&output)["schemaVersion"], 4);
}

#[cfg(unix)]
#[test]
fn discovery_skips_symlink_cycles_and_deduplicates_explicit_file_symlinks() {
    let p = Project::new();
    p.file("models/a.sysml", "package P;");
    std::os::unix::fs::symlink(&p.0, p.0.join("models/loop")).unwrap();
    std::os::unix::fs::symlink(p.0.join("models/a.sysml"), p.0.join("alias.sysml")).unwrap();
    let output = p.run(&["parse", "--format=json", "models", "alias.sysml"], None);
    status(&output, 0);
    assert_eq!(report(&output)["summary"]["files"], 1);
}
