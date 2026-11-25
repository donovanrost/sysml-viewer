import assert from 'node:assert/strict';
import { cpSync, copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { join } from 'node:path';
import { cargo, root } from './rust.mjs';
import { verifyVendor } from './lib/vendor.mjs';

// Compile the vendored private regression tests in a disposable workspace.
// Cargo must never write a lockfile or build products into the pinned inventory.
verifyVendor(root);
const packages = (path) =>
  readFileSync(join(root, path), 'utf8')
    .split('[[package]]')
    .slice(1)
    .map((p) => p.trim());
const pinned = new Set(packages('Cargo.lock'));
assert(
  packages('tools/vendor-tests.Cargo.lock').every((p) => pinned.has(p)),
  'Vendor test lock must be a subset of the application dependency lock; review and regenerate it after upgrades',
);
const directory = join(root, 'target', 'vendor-tests');
mkdirSync(directory, { recursive: true });
const workspace = mkdtempSync(join(directory, 'source-'));
try {
  cpSync(join(root, 'vendor', 'sysml-toolkit'), workspace, { recursive: true });
  cpSync(
    join(root, 'vendor', 'sysml-library', 'sources'),
    join(workspace, 'spec-refs', 'SysML-v2-Release', 'sysml.library'),
    { recursive: true },
  );
  copyFileSync(join(root, 'tools', 'vendor-tests.Cargo.lock'), join(workspace, 'Cargo.lock'));
  const suites = [
    ['sysmlv2-syntax', 'lexer::'],
    ['sysmlv2-model', 'rational::tests::'],
    ['sysmlv2-model', 'check::typing::tests::'],
    ['sysmlv2-model', 'check::scalars::tests::'],
    ['sysmlv2-model', 'check::actions::transition_tests::'],
    ['sysmlv2-model', 'check::structural::view_rendering_tests::'],
    ['sysmlv2-model', 'json::indexed_reads_tests::an_unsettled_table_is_scanned'],
    ...[
      'type_relations',
      'type_features',
      'type_inputs',
      'membership_projection',
      'positional',
      'positional_names',
      'expression_results',
      'reference_memberships',
      'trigger_arguments',
      'feature_type_projection',
      'step_context',
      'implied',
      'structural_index',
      'model_level_evaluability',
      'cardinality',
      'invocation_bindings',
      'invocation_defaults',
      'invocation_results',
      'constructor_bindings',
      'dynamic_graph',
      'dynamic_invocations',
      'owned_results',
      'result_inheritance',
      'result_redefinition',
      'accept_bindings',
      'semantic_ownership',
      'semantic::certified_types',
      'end_constancy',
      'operations::namespaces',
    ].map((module) => ['sysmlv2-model', `json::${module}::`]),
  ];
  for (const [packageName, filter] of suites) {
    cargo(
      [
        'test',
        '--locked',
        '--release',
        '--manifest-path',
        join(workspace, 'Cargo.toml'),
        '--package',
        packageName,
        '--lib',
        filter,
        '--',
        '--test-threads=1',
      ],
      {
        env: {
          ...process.env,
          CARGO_TARGET_DIR: join(directory, 'build'),
          RUST_MIN_STACK: '16777216',
        },
      },
    );
  }
} finally {
  rmSync(workspace, { recursive: true, force: true });
}
