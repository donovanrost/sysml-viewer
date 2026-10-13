import init, { analyze } from '../../packages/kernel-wasm/sysml_wasm';
import wasmUrl from '../../packages/kernel-wasm/sysml_wasm_bg.wasm?url';
import type { AnalysisRequest } from './generated/AnalysisRequest';

self.onmessage = async (event: MessageEvent<AnalysisRequest>) => {
  try {
    const response = await fetch(wasmUrl);
    if (!response.ok) throw new Error(`Could not load language engine (${response.status})`);
    // Bytes avoid depending on a desktop custom protocol's WASM MIME type.
    await init({ module_or_path: await response.arrayBuffer() });
    const snapshot = JSON.parse(analyze(JSON.stringify(event.data)));
    self.postMessage({ revision: event.data.revision, snapshot });
  } catch (error) {
    self.postMessage({ revision: event.data.revision, error: String(error) });
  }
};
