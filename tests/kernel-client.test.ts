import { test } from 'node:test';
import type { TestContext } from 'node:test';
import assert from 'node:assert/strict';
import { KernelClient } from '../src/kernel/client.ts';

// A deterministic transport double: the assertions exercise the client's public
// promises/lifecycle. Actual WASM execution is covered by kernel.test.ts.
class TestWorker {
  static instances: TestWorker[] = [];
  terminated = false;
  onmessage?: (event: { data: unknown }) => void;
  onerror?: () => void;
  constructor() {
    TestWorker.instances.push(this);
  }
  postMessage() {}
  terminate() {
    this.terminated = true;
  }
  respond(revision: number, snapshotRevision = revision, schemaVersion = 3) {
    this.onmessage?.({
      data: { revision, snapshot: { revision: snapshotRevision, schemaVersion } },
    });
  }
}

function installWorker(t: TestContext) {
  const original = Object.getOwnPropertyDescriptor(globalThis, 'Worker');
  Object.defineProperty(globalThis, 'Worker', { configurable: true, value: TestWorker });
  TestWorker.instances = [];
  t.after(() => {
    if (original) Object.defineProperty(globalThis, 'Worker', original);
    else Reflect.deleteProperty(globalThis, 'Worker');
  });
  const client = new KernelClient();
  t.after(() => client.cancel());
  return client;
}

test('superseded analysis rejects and a late reply cannot replace the new result', async (t) => {
  const client = installWorker(t);
  const first = client.analyze([], 1);
  const aborted = assert.rejects(first, { name: 'AbortError' });
  const second = client.analyze([], 2);
  assert(TestWorker.instances[0].terminated);
  TestWorker.instances[0].respond(1);
  TestWorker.instances[1].respond(2);
  assert.equal((await second).revision, 2);
  await aborted;
});

test('snapshot revision and schema mismatches fail closed', async (t) => {
  const client = installWorker(t);
  const mismatched = client.analyze([], 3);
  TestWorker.instances.at(-1)!.respond(3, 2);
  await assert.rejects(mismatched, /revision/);
  const unknownSchema = client.analyze([], 4);
  TestWorker.instances.at(-1)!.respond(4, 4, 2);
  await assert.rejects(unknownSchema, /protocol/);
});

test('a worker error is isolated and the next request gets a fresh worker', async (t) => {
  const client = installWorker(t);
  const failed = client.analyze([], 5);
  TestWorker.instances.at(-1)!.onerror!();
  await assert.rejects(failed, /engine failed/);
  assert(TestWorker.instances[0].terminated);
  const recovered = client.analyze([], 6);
  TestWorker.instances.at(-1)!.respond(6);
  assert.equal((await recovered).revision, 6);
  assert.equal(TestWorker.instances.length, 2);
});

test('analysis deadline terminates the worker and rejects its promise', async (t) => {
  const client = installWorker(t);
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const request = client.analyze([], 7);
  const timedOut = assert.rejects(request, /60 seconds/);
  t.mock.timers.tick(60_000);
  await timedOut;
  assert(TestWorker.instances[0].terminated);
});
