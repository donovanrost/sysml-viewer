import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import init, { analyze } from '../packages/kernel-wasm/sysml_wasm.js';
import { assessCase, type ConformanceCheck } from '../tools/lib/conformance.mjs';
import type { Snapshot } from '../src/kernel/generated/Snapshot';

const read = (path: string) => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const matrix = JSON.parse(read('tests/conformance/matrix.json')) as {
  cases: { id: string; files: string[]; checks: ConformanceCheck[] }[];
};
const cases = matrix.cases.filter(
  (c) => c.id.startsWith('FAMILY-') || ['TYPE-005', 'TYPE-006', 'TYPE-007'].includes(c.id),
);
const documents = [...new Set(cases.flatMap((c) => c.files))].map((file) => ({
  id: file,
  name: file.split('/').at(-1)!,
  text: read(file),
}));

test('all declaration families retain native/WASM parity and their assertions in either file order', async () => {
  await init({
    module_or_path: readFileSync(
      new URL('../packages/kernel-wasm/sysml_wasm_bg.wasm', import.meta.url),
    ),
  });
  const executable = fileURLToPath(
    new URL(
      `../target/release/sysml-inspect${process.platform === 'win32' ? '.exe' : ''}`,
      import.meta.url,
    ),
  );
  for (const ordered of [documents, [...documents].reverse()]) {
    const request = JSON.stringify({ revision: 47, documents: ordered });
    const native: Snapshot = JSON.parse(
      execFileSync(executable, ['--request-json'], {
        input: request,
        encoding: 'utf8',
        timeout: 30_000,
        maxBuffer: 50_000_000,
      }),
    );
    const wasm: Snapshot = JSON.parse(analyze(request));
    assert.deepEqual(wasm, native);
    for (const c of cases) {
      // Each native matrix case normally runs alone. Keep its diagnostic scope
      // in this combined workspace, so deliberately invalid peer files do not
      // fail a positive file's noDiagnostics assertion.
      const scoped = {
        ...wasm,
        diagnostics: wasm.diagnostics.filter((d) => c.files.includes(d.source.documentId)),
      };
      assert.deepEqual(
        assessCase(scoped, c).filter((check) => check.status !== 'pass'),
        [],
        c.id,
      );
    }
  }
});
