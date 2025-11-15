// Hierarchical segment constraint validation

export interface TimeSegment {
  start: number;
  end: number;
}

/**
 * Validates hierarchical constraints for verse/word/anti-pattern segments
 */
export class SegmentConstraints {
  /**
   * Validate verse segment
   * - Must be within trim bounds
   * - Cannot overlap with other verses
   */
  static validateVerse(
    existing: Array<{ ayah: number } & TimeSegment>,
    proposed: { ayah: number } & TimeSegment,
    bounds: TimeSegment
  ): string[] {
    const errors: string[] = [];

    // Basic validation
    if (proposed.start >= proposed.end) {
      errors.push(`Invalid segment: start ${proposed.start.toFixed(2)} >= end ${proposed.end.toFixed(2)}`);
    }

    // Within bounds
    if (proposed.start < bounds.start) {
      errors.push(`Verse start ${proposed.start.toFixed(2)}s before trim start ${bounds.start.toFixed(2)}s`);
    }
    if (proposed.end > bounds.end) {
      errors.push(`Verse end ${proposed.end.toFixed(2)}s after trim end ${bounds.end.toFixed(2)}s`);
    }

    // No overlap with other verses
    for (const v of existing) {
      if (v.ayah === proposed.ayah) continue; // Allow updating same ayah

      const overlap = !(proposed.end <= v.start || proposed.start >= v.end);
      if (overlap) {
        errors.push(
          `Overlaps with Ayah ${v.ayah} [${v.start.toFixed(2)}s, ${v.end.toFixed(2)}s]`
        );
      }
    }

    return errors;
  }

  /**
   * Validate word segment
   * - Must be within verse bounds
   * - Overlap with other words is ALLOWED (for tajweed rules spanning words)
   */
  static validateWord(
    verseBounds: TimeSegment,
    proposed: TimeSegment
  ): string[] {
    const errors: string[] = [];

    // Basic validation
    if (proposed.start >= proposed.end) {
      errors.push(`Invalid segment: start ${proposed.start.toFixed(2)} >= end ${proposed.end.toFixed(2)}`);
    }

    // Within verse bounds
    if (proposed.start < verseBounds.start) {
      errors.push(
        `Word start ${proposed.start.toFixed(2)}s before verse start ${verseBounds.start.toFixed(2)}s`
      );
    }
    if (proposed.end > verseBounds.end) {
      errors.push(
        `Word end ${proposed.end.toFixed(2)}s after verse end ${verseBounds.end.toFixed(2)}s`
      );
    }

    // Note: Overlap with other words is explicitly allowed
    return errors;
  }

  /**
   * Validate anti-pattern segment
   * - Must be within word bounds
   * - Overlap with other anti-patterns in same word is allowed
   */
  static validateAntiPattern(
    wordBounds: TimeSegment,
    proposed: TimeSegment
  ): string[] {
    const errors: string[] = [];

    // Basic validation
    if (proposed.start >= proposed.end) {
      errors.push(`Invalid segment: start ${proposed.start.toFixed(2)} >= end ${proposed.end.toFixed(2)}`);
    }

    // Within word bounds
    if (proposed.start < wordBounds.start) {
      errors.push(
        `Anti-pattern start ${proposed.start.toFixed(2)}s before word start ${wordBounds.start.toFixed(2)}s`
      );
    }
    if (proposed.end > wordBounds.end) {
      errors.push(
        `Anti-pattern end ${proposed.end.toFixed(2)}s after word end ${wordBounds.end.toFixed(2)}s`
      );
    }

    return errors;
  }

  /**
   * Check if two segments overlap
   */
  static overlaps(a: TimeSegment, b: TimeSegment): boolean {
    return !(a.end <= b.start || a.start >= b.end);
  }

  /**
   * Check if segment A contains segment B
   */
  static contains(outer: TimeSegment, inner: TimeSegment): boolean {
    return inner.start >= outer.start && inner.end <= outer.end;
  }

  /**
   * Get segment duration
   */
  static duration(seg: TimeSegment): number {
    return seg.end - seg.start;
  }

  /**
   * Format segment for display
   */
  static format(seg: TimeSegment): string {
    return `[${seg.start.toFixed(2)}s, ${seg.end.toFixed(2)}s]`;
  }
}
