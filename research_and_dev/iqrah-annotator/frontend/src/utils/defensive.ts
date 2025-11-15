/**
 * Defensive Programming Utilities
 * Safe accessors and validators to prevent runtime errors
 */

/**
 * Safely get the first element of an array
 * Returns undefined if array is empty or null
 */
export function safeFirst<T>(arr: T[] | null | undefined): T | undefined {
  if (!arr || arr.length === 0) return undefined;
  return arr[0];
}

/**
 * Safely get the last element of an array
 * Returns undefined if array is empty or null
 */
export function safeLast<T>(arr: T[] | null | undefined): T | undefined {
  if (!arr || arr.length === 0) return undefined;
  return arr[arr.length - 1];
}

/**
 * Safely get an element at a specific index
 * Returns undefined if index is out of bounds
 */
export function safeAt<T>(arr: T[] | null | undefined, index: number): T | undefined {
  if (!arr || index < 0 || index >= arr.length) return undefined;
  return arr[index];
}

/**
 * Safely get a property from an object
 * Returns undefined if object is null or property doesn't exist
 */
export function safeGet<T, K extends keyof T>(
  obj: T | null | undefined,
  key: K
): T[K] | undefined {
  if (!obj) return undefined;
  return obj[key];
}

/**
 * Safely get a nested property using a path
 * Example: safeGetNested(obj, 'user.profile.name')
 */
export function safeGetNested(
  obj: any,
  path: string
): any {
  if (!obj) return undefined;

  const keys = path.split('.');
  let current = obj;

  for (const key of keys) {
    if (current == null) return undefined;
    current = current[key];
  }

  return current;
}

/**
 * Ensure a value is a number, with fallback
 */
export function ensureNumber(value: any, fallback: number = 0): number {
  if (typeof value === 'number' && !isNaN(value)) {
    return value;
  }
  return fallback;
}

/**
 * Ensure a value is a string, with fallback
 */
export function ensureString(value: any, fallback: string = ''): string {
  if (typeof value === 'string') {
    return value;
  }
  return fallback;
}

/**
 * Ensure a value is an array, with fallback
 */
export function ensureArray<T>(value: any, fallback: T[] = []): T[] {
  if (Array.isArray(value)) {
    return value;
  }
  return fallback;
}

/**
 * Clamp a number between min and max
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * Check if a value is defined (not null or undefined)
 */
export function isDefined<T>(value: T | null | undefined): value is T {
  return value !== null && value !== undefined;
}

/**
 * Check if an array has elements
 */
export function hasElements<T>(arr: T[] | null | undefined): arr is T[] {
  return Array.isArray(arr) && arr.length > 0;
}

/**
 * Check if a string is non-empty
 */
export function isNonEmptyString(value: any): value is string {
  return typeof value === 'string' && value.length > 0;
}

/**
 * Safely parse JSON, returning fallback on error
 */
export function safeJsonParse<T>(json: string, fallback: T): T {
  try {
    return JSON.parse(json);
  } catch {
    return fallback;
  }
}

/**
 * Safely call a function, catching errors and returning fallback
 */
export function safeTry<T>(fn: () => T, fallback: T): T {
  try {
    return fn();
  } catch (error) {
    console.error('[safeTry] Function threw error:', error);
    return fallback;
  }
}

/**
 * Safely call an async function, catching errors and returning fallback
 */
export async function safeAsyncTry<T>(
  fn: () => Promise<T>,
  fallback: T
): Promise<T> {
  try {
    return await fn();
  } catch (error) {
    console.error('[safeAsyncTry] Async function threw error:', error);
    return fallback;
  }
}

/**
 * Assert that a value is defined, throwing a descriptive error if not
 * Use this for values that should never be null/undefined (programmer errors)
 */
export function assertDefined<T>(
  value: T | null | undefined,
  message: string
): asserts value is T {
  if (value === null || value === undefined) {
    throw new Error(`Assertion failed: ${message}`);
  }
}

/**
 * Assert that an array has elements
 */
export function assertHasElements<T>(
  arr: T[] | null | undefined,
  message: string
): asserts arr is T[] {
  if (!Array.isArray(arr) || arr.length === 0) {
    throw new Error(`Assertion failed: ${message}`);
  }
}

/**
 * Create a debounced version of a function
 * Useful for preventing too many rapid calls
 */
export function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delayMs: number
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout | null = null;

  return function debounced(...args: Parameters<T>) {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }

    timeoutId = setTimeout(() => {
      fn(...args);
      timeoutId = null;
    }, delayMs);
  };
}

/**
 * Create a throttled version of a function
 * Ensures function is called at most once per interval
 */
export function throttle<T extends (...args: any[]) => any>(
  fn: T,
  intervalMs: number
): (...args: Parameters<T>) => void {
  let lastCall = 0;

  return function throttled(...args: Parameters<T>) {
    const now = Date.now();
    if (now - lastCall >= intervalMs) {
      lastCall = now;
      fn(...args);
    }
  };
}

/**
 * Retry an async operation with exponential backoff
 */
export async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  baseDelayMs: number = 1000
): Promise<T> {
  let lastError: any;

  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error;

      if (attempt < maxRetries) {
        const delay = baseDelayMs * Math.pow(2, attempt);
        console.log(`[retryWithBackoff] Attempt ${attempt + 1} failed, retrying in ${delay}ms...`);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }

  throw lastError;
}

/**
 * Wait for a condition to be true, with timeout
 */
export async function waitFor(
  condition: () => boolean,
  timeoutMs: number = 5000,
  intervalMs: number = 100
): Promise<boolean> {
  const startTime = Date.now();

  while (!condition()) {
    if (Date.now() - startTime > timeoutMs) {
      return false;
    }
    await new Promise(resolve => setTimeout(resolve, intervalMs));
  }

  return true;
}
