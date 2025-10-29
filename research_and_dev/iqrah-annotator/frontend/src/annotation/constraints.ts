// src/annotation/constraints.ts
import type {
  Annotation,
  AnnotationSet,
  ConstraintTable,
  ValidationResult,
  ValidationIssue,
} from './types';

export const DEFAULT_CONSTRAINTS: ConstraintTable = {
  surah: { disallowOverlapWithSameKind: true, mustBeInsideKind: null },
  word:  { disallowOverlapWithSameKind: true, mustBeInsideKind: 'surah' },
  other: { disallowOverlapWithSameKind: false, mustBeInsideKind: null },
};

export function overlaps(a: Annotation, b: Annotation): boolean {
  const s = Math.max(a.start, b.start);
  const e = Math.min(a.end, b.end);
  return s < e;
}
export function contains(outer: Annotation, inner: Annotation): boolean {
  return outer.start <= inner.start && outer.end >= inner.end;
}
export function isValidRange(a: Annotation): boolean {
  return isFinite(a.start) && isFinite(a.end) && a.start >= 0 && a.end >= a.start;
}

export function validateAnnotation(
  candidate: Annotation,
  all: AnnotationSet,
  constraints: ConstraintTable = DEFAULT_CONSTRAINTS,
  ignoreId?: string
): ValidationResult {
  const issues: ValidationIssue[] = [];
  if (!isValidRange(candidate)) {
    issues.push({ code: 'INVALID_RANGE', message: 'Start/end are invalid.' });
    return { valid: false, issues };
  }
  const rule = constraints[candidate.kind];
  const others = Object.values(all).filter(a => a.id !== ignoreId && a.id !== candidate.id);

  if (rule?.disallowOverlapWithSameKind) {
    const conflicts = others.filter(o => o.kind === candidate.kind && overlaps(candidate, o));
    if (conflicts.length) {
      issues.push({
        code: 'OVERLAP_SAME_KIND',
        message: `${cap(candidate.kind)} regions cannot overlap.`,
        conflictingIds: conflicts.map(c => c.id),
      });
    }
  }

  if (rule?.mustBeInsideKind) {
    const parents = others.filter(o => o.kind === rule.mustBeInsideKind);
    const container = parents.find(p => contains(p, candidate));
    if (!parents.length) {
      issues.push({
        code: 'PARENT_NOT_FOUND',
        message: `No ${rule.mustBeInsideKind} region exists to contain this ${candidate.kind}.`,
      });
    } else if (!container) {
      issues.push({
        code: 'MUST_BE_INSIDE_PARENT',
        message: `${cap(candidate.kind)} must be fully inside a ${rule.mustBeInsideKind} region.`,
        conflictingIds: parents.map(p => p.id),
      });
    }
  }

  return { valid: issues.length === 0, issues };
}

function cap(s: string) { return s.charAt(0).toUpperCase() + s.slice(1); }
