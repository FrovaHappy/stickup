import { describe, it, expect, beforeEach } from 'vitest';
import {
  gridSize,
  setGridSize,
  sortOption,
  setSortOption,
  sortEmotesList,
  askBeforePaste,
  setAskBeforePaste,
  lastCopiedEmote,
  recordCopiedEmote,
  formatFilter,
  setFormatFilter,
  filterEmotesByFormat
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
    setFormatFilter('all');
  });

  it('updates and holds grid size', () => {
    let current: string | undefined;
    const unsub = gridSize.subscribe((v) => (current = v));
    expect(current).toBe('medium');

    setGridSize('small');
    expect(current).toBe('small');

    setGridSize('large');
    expect(current).toBe('large');
    unsub();
  });

  it('updates and holds sort option', () => {
    let current: string | undefined;
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

  it('manages askBeforePaste preference and persistence', () => {
    let current: boolean | undefined;
    const unsub = askBeforePaste.subscribe((v) => (current = v));
    expect(current).toBe(true);

    setAskBeforePaste(false);
    expect(current).toBe(false);

    setAskBeforePaste(true);
    expect(current).toBe(true);
    unsub();
  });

  it('records last copied emote accurately', () => {
    const emote = createDummyEmote({ id: 42, name: 'PepeJam', path: '/emotes/pepe.webp' });
    recordCopiedEmote(emote);

    let recorded: any;
    const unsub = lastCopiedEmote.subscribe((v) => (recorded = v));
    expect(recorded).not.toBeNull();
    expect(recorded?.id).toBe(42);
    expect(recorded?.name).toBe('PepeJam');
    expect(recorded?.path).toBe('/emotes/pepe.webp');
    expect(typeof recorded?.timestamp).toBe('number');
    unsub();
  });

  it('manages formatFilter preference and updates', () => {
    let current: string | undefined;
    const unsub = formatFilter.subscribe((v) => (current = v));
    expect(current).toBe('all');

    setFormatFilter('gif');
    expect(current).toBe('gif');

    setFormatFilter('static');
    expect(current).toBe('static');

    setFormatFilter('all');
    expect(current).toBe('all');
    unsub();
  });

  it('filters emotes correctly by format (all, gif, static)', () => {
    const e1 = createDummyEmote({ id: 1, name: 'Static1', format: 'webp', path: '/emotes/s1.webp' });
    const e2 = createDummyEmote({ id: 2, name: 'Gif1', format: 'gif', path: '/emotes/g1.gif' });
    const e3 = createDummyEmote({ id: 3, name: 'Static2', format: 'png', path: '/emotes/s2.png' });
    const e4 = createDummyEmote({ id: 4, name: 'Gif2', format: 'webp', path: '/emotes/g2.gif' });

    const all = filterEmotesByFormat([e1, e2, e3, e4], 'all');
    expect(all.map((e) => e.id)).toEqual([1, 2, 3, 4]);

    const gifs = filterEmotesByFormat([e1, e2, e3, e4], 'gif');
    expect(gifs.map((e) => e.id)).toEqual([2, 4]);

    const statics = filterEmotesByFormat([e1, e2, e3, e4], 'static');
    expect(statics.map((e) => e.id)).toEqual([1, 3]);
  });
});

