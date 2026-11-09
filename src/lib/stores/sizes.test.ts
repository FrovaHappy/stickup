import { describe, it, expect, beforeEach } from 'vitest';
import {
  defaultResolution,
  setDefaultResolution,
  customSizes,
  addCustomSize,
  removeCustomSize,
  resolveDefaultDimensions
} from './sizes';

describe('sizes store - favorite resolution & presets', () => {
  beforeEach(() => {
    localStorage.clear();
    customSizes.set([]);
    setDefaultResolution('original');
  });

  it('initializes defaultResolution as original', () => {
    let current: string | undefined;
    const unsub = defaultResolution.subscribe((v) => (current = v));
    expect(current).toBe('original');
    unsub();
  });

  it('updates and persists defaultResolution', () => {
    let current: string | undefined;
    const unsub = defaultResolution.subscribe((v) => (current = v));

    setDefaultResolution('48');
    expect(current).toBe('48');
    expect(localStorage.getItem('stickup_pref_default_resolution')).toBe('48');

    setDefaultResolution('128');
    expect(current).toBe('128');
    expect(localStorage.getItem('stickup_pref_default_resolution')).toBe('128');

    unsub();
  });

  it('resets defaultResolution to original if its custom size is removed', () => {
    const custom = addCustomSize(80, 80);
    expect(custom).not.toBeNull();
    const val = custom!.value;

    setDefaultResolution(val);
    let current: string | undefined;
    const unsub = defaultResolution.subscribe((v) => (current = v));
    expect(current).toBe(val);

    // Remove custom size
    removeCustomSize(val);
    expect(current).toBe('original');
    expect(localStorage.getItem('stickup_pref_default_resolution')).toBe('original');

    unsub();
  });

  it('resolves dimensions correctly for original, presets, and custom sizes', () => {
    const dummyEmote = { original_width: 300, original_height: 200 };

    // 1. Original
    const orig = resolveDefaultDimensions(dummyEmote, 'original');
    expect(orig).toEqual({ width: 300, height: 200, label: 'Original' });

    // 2. Default preset '48'
    const preset48 = resolveDefaultDimensions(dummyEmote, '48');
    expect(preset48).toEqual({ width: 48, height: 48, label: '48x48' });

    // 3. Default preset '128'
    const preset128 = resolveDefaultDimensions(dummyEmote, '128');
    expect(preset128).toEqual({ width: 128, height: 128, label: '128x128' });

    // 4. Custom preset
    const customList = [{ label: '96x96', value: 'custom_96x96', width: 96, height: 96 }];
    const resolvedCustom = resolveDefaultDimensions(dummyEmote, 'custom_96x96', customList);
    expect(resolvedCustom).toEqual({ width: 96, height: 96, label: '96x96' });

    // 5. Fallback for null emote
    const nullEmoteRes = resolveDefaultDimensions(null, 'original');
    expect(nullEmoteRes).toEqual({ width: 48, height: 48, label: '48×48' });
  });
});

