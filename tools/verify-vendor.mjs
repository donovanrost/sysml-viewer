import { fileURLToPath } from 'node:url';
import { verifyVendor } from './lib/vendor.mjs';
const result = verifyVendor(fileURLToPath(new URL('..', import.meta.url)), {
  apply: process.argv.includes('--apply'),
});
console.log(
  `Verified ${Object.keys(result.hashes).length} vendored files and ${result.inputs.length - 1} recorded patch(es).`,
);
