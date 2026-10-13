import type { Snapshot } from './generated/Snapshot';
import type { SourceDocument } from './generated/SourceDocument';

/** One disposable worker per request gives hard cancellation and trap isolation. */
export class KernelClient {
  private pending?: {
    worker: Worker;
    reject: (reason: Error) => void;
    timer: ReturnType<typeof setTimeout>;
  };

  cancel() {
    if (!this.pending) return;
    clearTimeout(this.pending.timer);
    this.pending.worker.terminate();
    this.pending.reject(new DOMException('Analysis superseded', 'AbortError'));
    this.pending = undefined;
  }

  analyze(documents: SourceDocument[], revision: number): Promise<Snapshot> {
    this.cancel();
    return new Promise((resolve, reject) => {
      const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
      const finish = () => {
        if (this.pending?.worker === worker) {
          clearTimeout(this.pending.timer);
          this.pending = undefined;
        }
        worker.terminate();
      };
      const timer = setTimeout(() => {
        finish();
        reject(new Error('Analysis exceeded 60 seconds. Try a smaller workspace.'));
      }, 60_000);
      this.pending = { worker, reject, timer };
      worker.onmessage = ({ data }) => {
        finish();
        if (data.revision !== revision) {
          reject(new Error('Unexpected analysis revision'));
          return;
        }
        if (data.error) {
          reject(new Error(data.error));
          return;
        }
        if (data.snapshot?.schemaVersion !== 3) {
          reject(new Error('Incompatible kernel protocol'));
          return;
        }
        if (data.snapshot.revision !== revision) {
          reject(new Error('Snapshot does not belong to this analysis revision'));
          return;
        }
        resolve(data.snapshot as Snapshot);
      };
      worker.onerror = () => {
        finish();
        reject(new Error('The language engine failed. Reload the files to start a fresh worker.'));
      };
      worker.postMessage({ revision, documents });
    });
  }
}
