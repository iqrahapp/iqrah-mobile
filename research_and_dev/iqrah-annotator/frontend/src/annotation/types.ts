// src/annotation/types.ts
export type AnnotationId = string;

export type AnnotationKind = 'surah' | 'word' | 'other';

export interface AnnotationMeta {
  label?: string;
  color?: string;
  alpha?: number;
  parentId?: AnnotationId | null;
  [k: string]: unknown;
}

export interface Annotation {
  id: AnnotationId;
  kind: AnnotationKind;
  start: number;
  end: number;
  meta: AnnotationMeta;
}

export type AnnotationSet = Record<AnnotationId, Annotation>;

export interface ConstraintRule {
  disallowOverlapWithSameKind?: boolean;
  mustBeInsideKind?: AnnotationKind | null;
}

export type ConstraintTable = Record<AnnotationKind, ConstraintRule>;

export interface ValidationIssue {
  code:
    | 'OVERLAP_SAME_KIND'
    | 'MUST_BE_INSIDE_PARENT'
    | 'PARENT_NOT_FOUND'
    | 'INVALID_RANGE';
  message: string;
  conflictingIds?: AnnotationId[];
}

export interface ValidationResult {
  valid: boolean;
  issues: ValidationIssue[];
}

export interface ExportPayload {
  version: 1;
  createdAt: string;
  annotations: Annotation[];
}
