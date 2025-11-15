/**
 * Audio coordinate conversion utilities
 *
 * Handles conversion between absolute (full recording) and relative (trimmed segment) timestamps
 *
 * FIX #13: Added branded types for type safety
 */

import type { Annotation } from '../annotation/types';

// FIX #13: Branded types to prevent mixing absolute and relative times
export type AbsoluteTime = number & { readonly __brand: 'absolute' };
export type RelativeTime = number & { readonly __brand: 'relative' };

export class AudioCoordinateConverter {
  constructor(private timeOffset: number) {}

  /**
   * Convert absolute time (full recording) to relative time (trimmed segment)
   */
  toRelative(absoluteTime: number): number {
    return absoluteTime - this.timeOffset;
  }

  /**
   * Convert relative time (trimmed segment) to absolute time (full recording)
   */
  toAbsolute(relativeTime: number): number {
    return relativeTime + this.timeOffset;
  }

  /**
   * Convert annotation coordinates between absolute and relative
   *
   * @param ann - Annotation to convert
   * @param toAbsolute - If true, convert to absolute; if false, convert to relative
   * @returns New annotation object with converted coordinates
   */
  convertAnnotation<T extends Pick<Annotation, 'start' | 'end'>>(
    ann: T,
    toAbsolute: boolean
  ): T {
    return {
      ...ann,
      start: toAbsolute ? this.toAbsolute(ann.start) : this.toRelative(ann.start),
      end: toAbsolute ? this.toAbsolute(ann.end) : this.toRelative(ann.end),
    };
  }

  /**
   * Update the time offset (when switching to a different audio segment)
   */
  setTimeOffset(offset: number): void {
    this.timeOffset = offset;
  }

  /**
   * Get the current time offset
   */
  getTimeOffset(): number {
    return this.timeOffset;
  }
}

// FIX #13: Helper functions for explicit type conversion
/**
 * Create an absolute time value with type safety
 */
export function absolute(time: number): AbsoluteTime {
  return time as AbsoluteTime;
}

/**
 * Create a relative time value with type safety
 */
export function relative(time: number): RelativeTime {
  return time as RelativeTime;
}

/**
 * Time range interfaces with branded types
 */
export interface AbsoluteTimeRange {
  start: AbsoluteTime;
  end: AbsoluteTime;
}

export interface RelativeTimeRange {
  start: RelativeTime;
  end: RelativeTime;
}

/**
 * Convert absolute time range to relative
 */
export function rangeToRelative(
  range: AbsoluteTimeRange,
  converter: AudioCoordinateConverter
): RelativeTimeRange {
  return {
    start: converter.toRelative(range.start) as RelativeTime,
    end: converter.toRelative(range.end) as RelativeTime,
  };
}

/**
 * Convert relative time range to absolute
 */
export function rangeToAbsolute(
  range: RelativeTimeRange,
  converter: AudioCoordinateConverter
): AbsoluteTimeRange {
  return {
    start: converter.toAbsolute(range.start) as AbsoluteTime,
    end: converter.toAbsolute(range.end) as AbsoluteTime,
  };
}
