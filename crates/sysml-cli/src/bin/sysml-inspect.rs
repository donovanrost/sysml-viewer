// Compatibility entry point for existing tooling consuming snapshot JSON.
fn main() -> std::process::ExitCode {
    sysml_cli::entry(true)
}
