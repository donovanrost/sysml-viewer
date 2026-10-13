import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { cpus, totalmem, release } from 'node:os';
import { createHash } from 'node:crypto';
import { cargo, root } from './rust.mjs';
import { assessCase } from './lib/conformance.mjs';

// Each WASM sample gets a fresh Node process and WebAssembly instance, matching
// the disposable-worker policy. The measured process also includes V8 overhead.
if (process.argv.includes('--wasm-child')) {
  const { default: init, analyze } = await import(
    pathToFileURL(join(root, 'packages/kernel-wasm/sysml_wasm.js'))
  );
  await init({
    module_or_path: readFileSync(join(root, 'packages/kernel-wasm/sysml_wasm_bg.wasm')),
  });
  process.stdout.write(analyze(readFileSync(0, 'utf8')));
} else {
  cargo(['build', '--locked', '--release', '-p', 'sysml-cli']);
  execFileSync(process.execPath, [join(root, 'tools/build-kernel.mjs')], { stdio: 'inherit' });
  const binary = join(
    root,
    `target/release/sysml-inspect${process.platform === 'win32' ? '.exe' : ''}`,
  );
  const scenarios = [
    {
      name: 'compound-ranges',
      text: readFileSync(join(root, 'tests/conformance/fixtures/mult-022.sysml'), 'utf8'),
    },
    {
      name: 'isolated-providers',
      text: readFileSync(join(root, 'tests/conformance/fixtures/mult-020.sysml'), 'utf8'),
      checks: JSON.parse(
        readFileSync(join(root, 'tests/conformance/matrix.json'), 'utf8'),
      ).cases.find((c) => c.id === 'MULT-020').checks,
    },
    {
      name: 'import-closures',
      text: readFileSync(join(root, 'tests/conformance/fixtures/imp-020.sysml'), 'utf8'),
    },
    {
      name: 'viewer-example',
      text: readFileSync(join(root, 'examples/CoastalExplorer.sysml'), 'utf8'),
    },
    {
      name: 'attribute-providers',
      text: readFileSync(join(root, 'tests/conformance/fixtures/mult-011.sysml'), 'utf8'),
    },
    {
      name: 'arithmetic-providers',
      text: readFileSync(join(root, 'tests/conformance/fixtures/mult-013.sysml'), 'utf8'),
    },
    {
      name: 'conditional-providers',
      text: readFileSync(join(root, 'tests/conformance/fixtures/mult-017.sysml'), 'utf8'),
    },
  ];
  for (const count of [100, 1000])
    scenarios.push({
      name: `synthetic-${count}-usages`,
      text: `package Scale { part def Component { attribute mass : ScalarValues::Real; port p; } part assembly { ${Array.from({ length: count }, (_, i) => `part c${i} : Component;`).join('\n')} } }`,
    });
  const results = [];
  let metadata;
  for (const scenario of scenarios) {
    const request = JSON.stringify({
      revision: 1,
      documents: [{ id: 'benchmark', name: 'Benchmark.sysml', text: scenario.text }],
    });
    let native;
    for (const mode of ['native', 'wasm']) {
      const samples = [];
      for (let sample = 0; sample < 3; sample++) {
        let command = mode === 'native' ? binary : process.execPath;
        let args =
          mode === 'native' ? ['--request-json'] : [fileURLToPath(import.meta.url), '--wasm-child'];
        const measured =
          ['darwin', 'linux'].includes(process.platform) && existsSync('/usr/bin/time');
        if (measured) {
          args = [process.platform === 'darwin' ? '-l' : '-v', command, ...args];
          command = '/usr/bin/time';
        }
        const started = performance.now();
        const processResult = spawnSync(command, args, {
          input: request,
          encoding: 'utf8',
          timeout: 60_000,
          maxBuffer: 100_000_000,
        });
        if (processResult.error) throw processResult.error;
        assert.equal(processResult.status, 0, processResult.stderr);
        const output = processResult.stdout;
        const elapsedMs = Math.round(performance.now() - started);
        const snapshot = JSON.parse(output);
        metadata ??= {
          engineVersion: snapshot.engineVersion,
          libraryVersion: snapshot.libraryVersion,
          schemaVersion: snapshot.schemaVersion,
        };
        assert.equal(snapshot.engineVersion, metadata.engineVersion);
        if (scenario.checks) {
          const failures = assessCase(snapshot, { checks: scenario.checks }).filter(
            (c) => c.status !== 'pass',
          );
          assert.deepEqual(failures, [], `${scenario.name}: checked isolation regression`);
        } else {
          assert.deepEqual(snapshot.diagnostics, [], `${scenario.name}: unexpected diagnostics`);
        }
        if (mode === 'native') native ??= snapshot;
        assert.deepEqual(snapshot, native, `${scenario.name}: repeat or native/WASM mismatch`);
        let peakRssBytes = null;
        if (measured) {
          const stats = processResult.stderr;
          const match =
            process.platform === 'darwin'
              ? stats.match(/(\d+)\s+maximum resident set size/)
              : stats.match(/Maximum resident set size \(kbytes\):\s*(\d+)/i);
          assert(match, 'Could not read peak RSS from time output');
          peakRssBytes = Number(match[1]) * (process.platform === 'linux' ? 1024 : 1);
        }
        samples.push({
          elapsedMs,
          peakRssBytes,
          outputBytes: Buffer.byteLength(output),
          elements: snapshot.elements.length,
        });
        console.log(
          `${scenario.name} ${mode} ${sample + 1}/3: ${elapsedMs} ms, peak RSS ${peakRssBytes === null ? 'unavailable' : (peakRssBytes / 1048576).toFixed(1) + ' MiB'}`,
        );
      }
      results.push({
        scenario: scenario.name,
        mode,
        sourceBytes: Buffer.byteLength(scenario.text),
        sourceSha256: createHash('sha256').update(scenario.text).digest('hex'),
        medianElapsedMs: samples.map((s) => s.elapsedMs).sort((a, b) => a - b)[1],
        maxPeakRssBytes: samples.every((s) => s.peakRssBytes === null)
          ? null
          : Math.max(...samples.map((s) => s.peakRssBytes ?? 0)),
        samples,
      });
    }
  }
  const report = {
    generatedAt: new Date().toISOString(),
    platform: process.platform,
    arch: process.arch,
    osRelease: release(),
    cpu: cpus()[0]?.model,
    logicalCpus: cpus().length,
    systemMemoryBytes: totalmem(),
    ...metadata,
    methodology:
      'Three fresh processes per scenario/transport. Wall time includes process startup, WASM initialization, analysis and JSON serialization/output. Peak RSS covers the entire process (Node/V8 for WASM). These synthetic workloads are not production-model or browser-latency promises. No warm-cache or incremental claim.',
    results,
  };
  mkdirSync(join(root, 'target/benchmarks'), { recursive: true });
  writeFileSync(
    join(root, 'target/benchmarks/latest.json'),
    JSON.stringify(report, null, 2) + '\n',
  );
}
