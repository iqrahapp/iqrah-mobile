import { describe, it, expect, beforeEach } from 'vitest';
import { stripHtml, clearHtmlStripCache, getHtmlStripCacheStats } from './utils';

describe('utils', () => {
  beforeEach(() => {
    clearHtmlStripCache();
  });

  describe('stripHtml', () => {
    it('should strip simple HTML tags', () => {
      const html = '<p>Hello World</p>';
      const result = stripHtml(html);

      expect(result).toBe('Hello World');
    });

    it('should strip nested HTML tags', () => {
      const html = '<div><span><strong>Bold Text</strong></span></div>';
      const result = stripHtml(html);

      expect(result).toBe('Bold Text');
    });

    it('should preserve Arabic text', () => {
      const html = '<span class="tajweed">بِسْمِ اللَّهِ</span>';
      const result = stripHtml(html);

      expect(result).toBe('بِسْمِ اللَّهِ');
    });

    it('should handle multiple elements', () => {
      const html = '<p>First</p><p>Second</p>';
      const result = stripHtml(html);

      expect(result).toBe('FirstSecond');
    });

    it('should handle empty string', () => {
      const result = stripHtml('');

      expect(result).toBe('');
    });

    it('should handle plain text without tags', () => {
      const text = 'Plain text';
      const result = stripHtml(text);

      expect(result).toBe('Plain text');
    });

    it('should handle HTML entities', () => {
      const html = '<p>&lt;Hello&gt; &amp; &quot;World&quot;</p>';
      const result = stripHtml(html);

      expect(result).toBe('<Hello> & "World"');
    });

    it('should handle self-closing tags', () => {
      const html = 'Line 1<br/>Line 2<hr/>Line 3';
      const result = stripHtml(html);

      expect(result).toBe('Line 1Line 2Line 3');
    });

    it('should handle attributes', () => {
      const html = '<a href="https://example.com" class="link">Link</a>';
      const result = stripHtml(html);

      expect(result).toBe('Link');
    });

    it('should handle special characters in text', () => {
      const html = '<p>Price: $100 &lt; $200</p>';
      const result = stripHtml(html);

      expect(result).toBe('Price: $100 < $200');
    });
  });

  describe('stripHtml caching', () => {
    it('should cache results', () => {
      const html = '<p>Cached Text</p>';

      // First call
      const result1 = stripHtml(html);
      const stats1 = getHtmlStripCacheStats();

      // Second call (should hit cache)
      const result2 = stripHtml(html);
      const stats2 = getHtmlStripCacheStats();

      expect(result1).toBe(result2);
      expect(stats1.size).toBe(1);
      expect(stats2.size).toBe(1); // Same cache size
    });

    it('should implement LRU eviction when cache is full', () => {
      // Fill cache to capacity (200 entries as per implementation)
      for (let i = 0; i < 205; i++) {
        stripHtml(`<p>Text ${i}</p>`);
      }

      const stats = getHtmlStripCacheStats();

      // Should have evicted oldest entries
      expect(stats.size).toBeLessThanOrEqual(200);
    });

    it('should clear cache', () => {
      stripHtml('<p>Test 1</p>');
      stripHtml('<p>Test 2</p>');

      let stats = getHtmlStripCacheStats();
      expect(stats.size).toBe(2);

      clearHtmlStripCache();

      stats = getHtmlStripCacheStats();
      expect(stats.size).toBe(0);
    });

    it('should move accessed items to end (LRU)', () => {
      // Add multiple items
      stripHtml('<p>Item 1</p>');
      stripHtml('<p>Item 2</p>');
      stripHtml('<p>Item 3</p>');

      // Access first item again (should move to end)
      stripHtml('<p>Item 1</p>');

      // Cache should still have all 3 items
      const stats = getHtmlStripCacheStats();
      expect(stats.size).toBe(3);
    });
  });

  describe('getHtmlStripCacheStats', () => {
    it('should return correct stats', () => {
      stripHtml('<p>Test 1</p>');
      stripHtml('<p>Test 2</p>');

      const stats = getHtmlStripCacheStats();

      expect(stats.size).toBe(2);
      expect(stats.maxSize).toBe(200);
      expect(stats.utilizationPercent).toBe(1); // 2/200 * 100 = 1%
    });

    it('should start with empty stats', () => {
      const stats = getHtmlStripCacheStats();

      expect(stats.size).toBe(0);
      expect(stats.utilizationPercent).toBe(0);
    });
  });

  describe('stripHtml edge cases', () => {
    it('should handle malformed HTML', () => {
      const html = '<p>Unclosed paragraph';
      const result = stripHtml(html);

      expect(result).toBe('Unclosed paragraph');
    });

    it('should handle script tags', () => {
      const html = '<script>alert("XSS")</script><p>Safe Text</p>';
      const result = stripHtml(html);

      // Script content should be removed
      expect(result).not.toContain('alert');
      expect(result).toContain('Safe Text');
    });

    it('should handle style tags', () => {
      const html = '<style>.class { color: red; }</style><p>Styled Text</p>';
      const result = stripHtml(html);

      // Style content should be removed
      expect(result).not.toContain('color');
      expect(result).toContain('Styled Text');
    });

    it('should handle comments', () => {
      const html = '<!-- Comment --><p>Text</p>';
      const result = stripHtml(html);

      expect(result).not.toContain('Comment');
      expect(result).toBe('Text');
    });

    it('should handle deeply nested structures', () => {
      const html = '<div><div><div><div><div>Deep</div></div></div></div></div>';
      const result = stripHtml(html);

      expect(result).toBe('Deep');
    });

    it('should handle mixed content', () => {
      const html = 'Text before <strong>bold</strong> text after';
      const result = stripHtml(html);

      expect(result).toBe('Text before bold text after');
    });

    it('should handle unicode characters', () => {
      const html = '<p>Unicode: 你好 مرحبا 🌟</p>';
      const result = stripHtml(html);

      expect(result).toBe('Unicode: 你好 مرحبا 🌟');
    });

    it('should handle whitespace preservation', () => {
      const html = '<p>Line 1</p>\n<p>Line 2</p>';
      const result = stripHtml(html);

      // DOMParser may normalize whitespace differently
      expect(result).toContain('Line 1');
      expect(result).toContain('Line 2');
    });
  });
});
