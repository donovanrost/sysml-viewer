use sysml_kernel::{AnalysisRequest, Snapshot};
use ts_rs::{Config, TS};

fn main() {
    let cfg = Config::from_env();
    Snapshot::export_all(&cfg).expect("export snapshot contract");
    AnalysisRequest::export_all(&cfg).expect("export request contract");
}
