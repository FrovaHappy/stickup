import { describe, it, expect, beforeEach } from 'vitest';
import {
  gridSize,
  setGridSize,
  sortOption,
  setSortOption,
  sortEmotesList
} from './preferences';
import type { Emote } from '$lib/types/emote';

function createDummyEmote(overrides: Partial<Emote>): Emote {
  return {
    id: 1,
    name: 'emote',
    path: '/path/test.webp',
    original_width: 128,
    original_height: 128,
    format: 'webp',
    created_at: '2026-01-01T00:00:00Z',
    copy_count: 0,
    interaction_count: 0,
    ...overrides
  };
}

describe('preferences store and sorting helpers', () => {
  beforeEach(() => {
    setGridSize('medium');
    setSortOption('date-desc');
  });

  it('updates and holds grid size', () => {
    let current;
    const unsub = gridSize.subscribe((v) => (current = v));
    expect(current).toBe('medium');

    setGridSize('small');
    expect(current).toBe('small');

    setGridSize('large');
    expect(current).toBe('large');
    unsub();
  });

  it('updates and holds sort option', () => {
    let current;
    const unsub = sortOption.subscribe((v) => (current = v));
    expect(current).toBe('date-desc');

    setSortOption('name-asc');
    expect(current).toBe('name-asc');
    unsub();
  });

  it('sorts emotes alphabetically (name-asc & name-desc)', () => {
    const e1 = createDummyEmote({ id: 1, name: 'Zebra' });
    const e2 = createDummyEmote({ id: 2, name: 'Apple' });
    const e3 = createDummyEmote({ id: 3, name: 'banana' });

    const asc = sortEmotesList([e1, e2, e3], 'name-asc');
    expect(asc.map((e) => e.name)).toEqual(['Apple', 'banana', 'Zebra']);

    const desc = sortEmotesList([e1, e2, e3], 'name-desc');
    expect(desc.map((e) => e.name)).toEqual(['Zebra', 'banana', 'Apple']);
  });

  it('sorts emotes by date (date-desc & date-asc)', () => {
    const e1 = createDummyEmote({ id: 1, created_at: '2026-01-01T00:00:00Z' });
    const e2 = createDummyEmote({ id: 2, created_at: '2026-03-01T00:00:00Z' });
    const e3 = createDummyEmote({ id: 3, created_at: '2026-02-01T00:00:00Z' });

    const desc = sortEmotesList([e1, e2, e3], 'date-desc');
    expect(desc.map((e) => e.id)).toEqual([2, 3, 1]);

    const asc = sortEmotesList([e1, e2, e3], 'date-asc');
    expect(asc.map((e) => e.id)).toEqual([1, 3, 2]);
  });

  it('sorts emotes by popularity / copy count (copies-desc)', () => {
    const e1 = createDummyEmote({ id: 1, copy_count: 5 });
    const e2 = createDummyEmote({ id: 2, copy_count: 100 });
    const e3 = createDummyEmote({ id: 3, copy_count: 20 });

    const sorted = sortEmotesList([e1, e2, e3], 'copies-desc');
    expect(sorted.map((e) => e.id)).toEqual([2, 3, 1]);
  });
});

