import { join } from 'node:path';
import { mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs';
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
