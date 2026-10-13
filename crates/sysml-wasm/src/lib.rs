use sysml_kernel::{AnalysisRequest, Engine};
use wasm_bindgen::prelude::*;

/// Thin transport binding. The CLI and browser execute the same analysis function.
#[wasm_bindgen]
pub fn analyze(request_json: &str) -> Result<String, String> {
    if request_json.len() > 25_000_000 {
        return Err("Request exceeds 25 MB".into());
    }
    let request: AnalysisRequest =
        serde_json::from_str(request_json).map_err(|e| format!("Invalid request: {e}"))?;
    let snapshot = Engine::bundled()
        .analyze(request, || false)
        .map_err(|e| e.to_string())?;
    serde_json::to_string(&snapshot).map_err(|e| e.to_string())
}
