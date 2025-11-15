/**
 * Shared utility functions
 */

// FIX #15: Cache for HTML stripping with LRU eviction
const htmlStripCache = new Map<string, string>();
const MAX_CACHE_SIZE = 200; // Increased from 100

// FIX #15: Use DOMParser instead of createElement for better performance
const parser = new DOMParser();

/**
 * Strips HTML tags from a string using DOMParser (FIX #15)
 * Memoized with LRU cache to avoid repeated parsing
 *
 * Performance improvements:
 * - DOMParser instead of createElement (faster, no DOM insertion)
 * - LRU cache eviction (preserve most recent entries)
 * - Increased cache size to 200 entries
 *
 * @param html - HTML string to strip
 * @returns Plain text content
 */
export function stripHtml(html: string): string {
  // Check cache first
  if (htmlStripCache.has(html)) {
    const cached = htmlStripCache.get(html)!;
    // Move to end (LRU: most recently used)
    htmlStripCache.delete(html);
    htmlStripCache.set(html, cached);
    return cached;
  }

  // FIX #15: Use DOMParser (more efficient than createElement)
  const doc = parser.parseFromString(html, 'text/html');

  // Remove script and style tags (security and cleanliness)
  doc.querySelectorAll('script, style').forEach(el => el.remove());

  const result = doc.body.textContent || '';

  // LRU eviction: remove oldest entry when cache is full
  if (htmlStripCache.size >= MAX_CACHE_SIZE) {
    // First entry is the oldest (Map preserves insertion order)
    const oldestKey = htmlStripCache.keys().next().value;
    htmlStripCache.delete(oldestKey);
  }

  htmlStripCache.set(html, result);

  return result;
}

/**
 * Clears the HTML strip cache (useful for testing or memory management)
 */
export function clearHtmlStripCache(): void {
  htmlStripCache.clear();
}

/**
 * Get cache statistics (for debugging)
 */
export function getHtmlStripCacheStats() {
  return {
    size: htmlStripCache.size,
    maxSize: MAX_CACHE_SIZE,
    utilizationPercent: (htmlStripCache.size / MAX_CACHE_SIZE) * 100,
  };
}
