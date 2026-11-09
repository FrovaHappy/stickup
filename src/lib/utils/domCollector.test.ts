import { describe, it, expect } from 'vitest';
import {
  parseCollectorInput,
  generateCollectorScript,
  generateBookmarklet
} from './domCollector';

describe('domCollector utils', () => {
  it('parses structured JSON payload with items and source', () => {
    const payload = JSON.stringify({
      stickup_collector_v1: true,
      source: 'https://twitch.tv/example',
      items: [
        { url: 'https://static-cdn.jtvnw.net/emoticons/v2/123/default/dark/3.0', alt: 'PogChamp' },
        { url: 'https://static-cdn.jtvnw.net/emoticons/v2/456/default/dark/3.0', alt: null }
      ]
    });

    const result = parseCollectorInput(payload);
    expect(result.source).toBe('https://twitch.tv/example');
    expect(result.items.length).toBe(2);
    expect(result.items[0]).toEqual({
      url: 'https://static-cdn.jtvnw.net/emoticons/v2/123/default/dark/3.0',
      alt: 'PogChamp'
    });
    expect(result.items[1]).toEqual({
      url: 'https://static-cdn.jtvnw.net/emoticons/v2/456/default/dark/3.0',
      alt: null
    });
  });

  it('parses raw array of URL strings', () => {
    const payload = JSON.stringify([
      'https://example.com/cat.png',
      'https://example.com/dog.webp'
    ]);

    const result = parseCollectorInput(payload);
    expect(result.items.length).toBe(2);
    expect(result.items[0].url).toBe('https://example.com/cat.png');
    expect(result.items[1].url).toBe('https://example.com/dog.webp');
  });

  it('parses plain text line-by-line URLs and ignores empty/invalid lines', () => {
    const text = `
      https://example.com/pepe.gif
      some random text that is not a url
      https://example.com/pepe.gif # duplicada
      https://example.com/frog.webp
    `;

    const result = parseCollectorInput(text);
    expect(result.items.length).toBe(2);
    expect(result.items[0].url).toBe('https://example.com/pepe.gif');
    expect(result.items[1].url).toBe('https://example.com/frog.webp');
  });

  it('handles empty input gracefully', () => {
    expect(parseCollectorInput('')).toEqual({ items: [] });
    expect(parseCollectorInput('   \n  ')).toEqual({ items: [] });
  });

  it('generates collector script with polling and deduplication', () => {
    const script = generateCollectorScript();
    expect(script).toContain('setInterval(scanDom, 1000)');
    expect(script).toContain('document.images');
    expect(script).toContain('collectedMap');
    expect(script).toContain('navigator.clipboard.writeText');
  });

  it('generates valid javascript: bookmarklet without encoding artifacts', () => {
    const bookmarklet = generateBookmarklet();
    expect(bookmarklet.startsWith('javascript:(function()')).toBe(true);
    expect(bookmarklet).not.toContain('%20');
    expect(bookmarklet).not.toContain('%0A');
    expect(bookmarklet).not.toContain('%7B');
    expect(bookmarklet.length).toBeGreaterThan(50);
  });
});

