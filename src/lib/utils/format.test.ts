import { describe, it, expect } from 'vitest';
import {
  isAnimatedEmote,
  getAvailableFormats,
  getMimeType,
  formatFileSize,
  resolveDimensions,
  toPascalCase,
  formatSearchQueryInput
} from './format';
import type { Emote } from '$lib/types/emote';

const baseEmote: Emote = {
  id: 1,
  name: 'pepe',
  tag: 'meme, frog',
  path: '/emotes/pepe.webp',
  original_width: 300,
  original_height: 200,
  format: 'webp',
  created_at: '2026-11-01T00:00:00Z'
};

describe('format utilities', () => {
  it('detects animated GIF emotes by format or extension', () => {
    expect(isAnimatedEmote(null)).toBe(false);
    expect(isAnimatedEmote(baseEmote)).toBe(false);
    expect(isAnimatedEmote({ ...baseEmote, format: 'gif' })).toBe(true);
    expect(isAnimatedEmote({ ...baseEmote, format: 'webp', path: '/emotes/dance.GIF' })).toBe(true);
  });

  it('returns correct available formats for static and animated emotes', () => {
    expect(getAvailableFormats(baseEmote)).toEqual(['webp', 'png', 'jpg']);
    expect(getAvailableFormats({ ...baseEmote, format: 'gif' })).toEqual(['gif']);
  });

  it('maps format strings to MIME types', () => {
    expect(getMimeType('webp')).toBe('image/webp');
    expect(getMimeType('jpg')).toBe('image/jpeg');
    expect(getMimeType('jpeg')).toBe('image/jpeg');
    expect(getMimeType('png')).toBe('image/png');
  });

  it('formats file sizes into human readable strings', () => {
    expect(formatFileSize(null)).toBe('Desconocido');
    expect(formatFileSize(0)).toBe('Desconocido');
    expect(formatFileSize(512)).toBe('512 B');
    expect(formatFileSize(2048)).toBe('2.0 KB');
    expect(formatFileSize(2 * 1024 * 1024)).toBe('2.0 MB');
  });

  it('resolves dimensions for original and preset sizes', () => {
    expect(resolveDimensions(null, 'original')).toEqual({ width: 48, height: 48 });
    expect(resolveDimensions(baseEmote, 'original')).toEqual({ width: 300, height: 200 });
    expect(resolveDimensions(baseEmote, '128')).toEqual({ width: 128, height: 128 });
    expect(resolveDimensions(baseEmote, 'unknown')).toEqual({ width: 300, height: 200 });
  });

  it('resolves dimensions for custom sizes and extra presets', () => {
    expect(resolveDimensions(baseEmote, 'custom_96x96')).toEqual({ width: 96, height: 96 });
    expect(resolveDimensions(baseEmote, '32x32')).toEqual({ width: 32, height: 32 });
    expect(
      resolveDimensions(baseEmote, 'my_special', [
        { label: 'Special', value: 'my_special', width: 110, height: 110 }
      ])
    ).toEqual({ width: 110, height: 110 });
  });

  describe('toPascalCase', () => {
    it('normalizes single words and lowercase to PascalCase', () => {
      expect(toPascalCase('pepe')).toBe('Pepe');
      expect(toPascalCase('cat')).toBe('Cat');
      expect(toPascalCase('frog')).toBe('Frog');
    });

    it('normalizes snake_case, kebab-case and spaces to PascalCase', () => {
      expect(toPascalCase('pepe_dance')).toBe('PepeDance');
      expect(toPascalCase('pepe-dance')).toBe('PepeDance');
      expect(toPascalCase('pepe dance')).toBe('PepeDance');
      expect(toPascalCase('cool__emote--super   fast')).toBe('CoolEmoteSuperFast');
    });

    it('preserves existing camelCase and PascalCase words', () => {
      expect(toPascalCase('pepeHands')).toBe('PepeHands');
      expect(toPascalCase('PogChamp')).toBe('PogChamp');
      expect(toPascalCase('monkaS')).toBe('MonkaS');
      expect(toPascalCase('catJam')).toBe('CatJam');
    });

    it('handles uppercase strings and tags', () => {
      expect(toPascalCase('OMEGALUL')).toBe('Omegalul');
      expect(toPascalCase('KEKW')).toBe('Kekw');
      expect(toPascalCase('TWITCH_EMOTE')).toBe('TwitchEmote');
    });

    it('handles alphanumeric tags and unicode accents', () => {
      expect(toPascalCase('tag1')).toBe('Tag1');
      expect(toPascalCase('emote_123_test')).toBe('Emote123Test');
      expect(toPascalCase('canción_triste')).toBe('CanciónTriste');
    });

    it('handles empty and null values gracefully', () => {
      expect(toPascalCase('')).toBe('');
      expect(toPascalCase('   ')).toBe('');
      expect(toPascalCase(null)).toBe('');
      expect(toPascalCase(undefined)).toBe('');
    });
  });

  describe('formatSearchQueryInput', () => {
    it('formats single words into PascalCase on the fly', () => {
      expect(formatSearchQueryInput('p')).toBe('P');
      expect(formatSearchQueryInput('pepe')).toBe('Pepe');
    });

    it('preserves trailing separator while typing next word', () => {
      expect(formatSearchQueryInput('Pepe ')).toBe('Pepe ');
      expect(formatSearchQueryInput('pepe_')).toBe('Pepe ');
      expect(formatSearchQueryInput('pepe-')).toBe('Pepe ');
    });

    it('merges previous words and capitalizes new word when character typed', () => {
      expect(formatSearchQueryInput('Pepe d')).toBe('PepeD');
      expect(formatSearchQueryInput('Pepe da')).toBe('PepeDa');
      expect(formatSearchQueryInput('Pepe dance')).toBe('PepeDance');
    });

    it('handles full pasted strings into clean PascalCase', () => {
      expect(formatSearchQueryInput('pepe hands dance')).toBe('PepeHandsDance');
    });

    it('handles empty or blank search inputs', () => {
      expect(formatSearchQueryInput('')).toBe('');
      expect(formatSearchQueryInput('   ')).toBe('');
      expect(formatSearchQueryInput('---')).toBe('');
    });
  });
});


