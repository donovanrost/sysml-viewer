import type { Snapshot } from '../../src/kernel/generated/Snapshot';

/** Checks are loaded from JSON; validateMatrix validates their operation-specific fields. */
export interface ConformanceCheck {
  id: string;
  op: string;
  [field: string]: unknown;
}

export interface CheckAssessment {
  id: string;
  status: 'pass' | 'unexpected-pass' | 'known-gap' | 'fail';
  issue?: string;
  message?: string;
}

export function assessCase(
  snapshot: Snapshot,
  item: {
    checks: ConformanceCheck[];
    knownFailures?: { check: string; issue: string; reason: string }[];
  },
): CheckAssessment[];
