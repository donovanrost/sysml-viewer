import { join } from 'node:path';
import { mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import assert from 'node:assert/strict';
import { cargo, root } from './rust.mjs';
const check = process.argv.includes('--check');
const committed = join(root, 'src/kernel/generated');
const output = check ? mkdtempSync(join(tmpdir(), 'sysml-contract-')) : committed;
try {
  cargo(
    ['run', '--locked', '-p', 'sysml-kernel', '--features', 'bindings', '--bin', 'export-types'],
    { env: { ...process.env, TS_RS_EXPORT_DIR: output } },
  );
  // ts-rs places trailing spaces before field comments. Normalize identically
  // in generation and checking so generated contracts obey repository hygiene.
  for (const file of readdirSync(output)) {
    const path = join(output, file);
    writeFileSync(path, readFileSync(path, 'utf8').replace(/[ \t]+$/gm, ''));
  }
  if (check) {
    assert.deepEqual(
      readdirSync(output).sort(),
      readdirSync(committed).sort(),
      'Generated contract file set differs',
    );
    for (const file of readdirSync(output))
      assert.equal(
        readFileSync(join(output, file), 'utf8'),
        readFileSync(join(committed, file), 'utf8'),
        `Regenerate kernel contract: ${file}`,
      );
  }
} finally {
  if (check) rmSync(output, { recursive: true, force: true });
}
