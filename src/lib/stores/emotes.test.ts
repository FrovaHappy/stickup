import { describe, it, expect } from 'vitest';
import { isEmoteNewByDate, compareEmotesByPopularity } from './emotes';
import type { Emote } from '$lib/types/emote';

function makeEmote(overrides: Partial<Emote>): Emote {
  return {
    id: 1,
    name: 'test',
    path: '/emotes/test.webp',
    original_width: 128,
    original_height: 128,
    format: 'webp',
    created_at: new Date().toISOString(),
    ...overrides
  };
}

describe('emotes store helpers', () => {
  it('identifies newly created emotes within 10 minutes', () => {
    expect(isEmoteNewByDate(null)).toBe(false);
    expect(isEmoteNewByDate('invalid-date')).toBe(false);
    expect(isEmoteNewByDate(new Date().toISOString())).toBe(true);

    const fiveMinutesAgo = new Date(Date.now() - 5 * 60 * 1000).toISOString();
    expect(isEmoteNewByDate(fiveMinutesAgo)).toBe(true);

    const elevenMinutesAgo = new Date(Date.now() - 11 * 60 * 1000).toISOString();
    expect(isEmoteNewByDate(elevenMinutesAgo)).toBe(false);
  });

  it('sorts emotes by copy_count descending, then last_used_at, then id', () => {
    const a = makeEmote({ id: 1, copy_count: 5, last_used_at: '2026-11-01T10:00:00Z' });
    const b = makeEmote({ id: 2, copy_count: 10, last_used_at: '2026-11-01T09:00:00Z' });
    const c = makeEmote({ id: 3, copy_count: 5, last_used_at: '2026-11-01T12:00:00Z' });
    const d = makeEmote({ id: 4, copy_count: 5, last_used_at: '2026-11-01T12:00:00Z' });

    const list = [a, b, d, c];
    list.sort(compareEmotesByPopularity);

    expect(list.map((e) => e.id)).toEqual([2, 3, 4, 1]);
  });
});

