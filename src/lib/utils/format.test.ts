import { describe, it, expect } from 'vitest';
import {
  isAnimatedEmote,
  getAvailableFormats,
  getMimeType,
  formatFileSize,
  fitDimensions,
  resolveDimensions,
  toPascalCase,
  formatSearchQueryInput,
  sanitizeEmoteName,
  parseUrlOrPath
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

  it('calculates proportional dimensions preserving aspect ratio with fitDimensions', () => {
    // Landscape (300x200) inside 128x128
    expect(fitDimensions(300, 200, 128, 128)).toEqual({ width: 128, height: 85 });
    // Portrait (200x300) inside 128x128
    expect(fitDimensions(200, 300, 128, 128)).toEqual({ width: 85, height: 128 });
    // Square (200x200) inside 128x128
    expect(fitDimensions(200, 200, 128, 128)).toEqual({ width: 128, height: 128 });
    // Custom rectangle bound (300x200) inside 200x100
    expect(fitDimensions(300, 200, 200, 100)).toEqual({ width: 150, height: 100 });
    // Edge cases
    expect(fitDimensions(0, 0, 128, 128)).toEqual({ width: 128, height: 128 });
    expect(fitDimensions(300, 200, 0, 0)).toEqual({ width: 300, height: 200 });
  });

  it('resolves dimensions preserving aspect ratio for presets and custom sizes', () => {
    expect(resolveDimensions(null, 'original')).toEqual({ width: 48, height: 48 });
    expect(resolveDimensions(baseEmote, 'original')).toEqual({ width: 300, height: 200 });
    // Landscape baseEmote (300x200) inside 128x128 bounding box => 128x85
    expect(resolveDimensions(baseEmote, '128')).toEqual({ width: 128, height: 85 });
    expect(resolveDimensions(baseEmote, 'unknown')).toEqual({ width: 300, height: 200 });

    // Square emote inside 128x128 => 128x128
    const squareEmote = { ...baseEmote, original_width: 256, original_height: 256 };
    expect(resolveDimensions(squareEmote, '128')).toEqual({ width: 128, height: 128 });

    // Portrait emote inside 128x128 => 85x128
    const portraitEmote = { ...baseEmote, original_width: 200, original_height: 300 };
    expect(resolveDimensions(portraitEmote, '128')).toEqual({ width: 85, height: 128 });
  });

  it('resolves dimensions preserving aspect ratio for custom sizes and extra presets', () => {
    // Landscape baseEmote (300x200) in custom_96x96 => 96x64
    expect(resolveDimensions(baseEmote, 'custom_96x96')).toEqual({ width: 96, height: 64 });
    // Landscape baseEmote (300x200) in 32x32 => 32x21
    expect(resolveDimensions(baseEmote, '32x32')).toEqual({ width: 32, height: 21 });
    // Landscape baseEmote (300x200) in 110x110 => 110x73
    expect(
      resolveDimensions(baseEmote, 'my_special', [
        { label: 'Special', value: 'my_special', width: 110, height: 110 }
      ])
    ).toEqual({ width: 110, height: 73 });
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

  describe('sanitizeEmoteName', () => {
    it('preserves spaces within name', () => {
      expect(sanitizeEmoteName('pepe dance')).toBe('pepe dance');
      expect(sanitizeEmoteName('Pepe Dance')).toBe('Pepe Dance');
      expect(sanitizeEmoteName('Cool Emote 123')).toBe('Cool Emote 123');
    });

    it('collapses multiple consecutive spaces and trims edges', () => {
      expect(sanitizeEmoteName('   pepe    dance   ')).toBe('pepe dance');
      expect(sanitizeEmoteName('  Cat   Jam  ')).toBe('Cat Jam');
    });

    it('removes illegal filesystem characters', () => {
      expect(sanitizeEmoteName('pepe/dance:cool*test?')).toBe('pepedancecooltest');
      expect(sanitizeEmoteName('emote<1>|"hi"')).toBe('emote1hi');
    });

    it('handles empty, whitespace or null inputs', () => {
      expect(sanitizeEmoteName('')).toBe('');
      expect(sanitizeEmoteName('   ')).toBe('');
      expect(sanitizeEmoteName(null)).toBe('');
      expect(sanitizeEmoteName(undefined)).toBe('');
    });
  });

  describe('parseUrlOrPath', () => {
    it('parses file:// URLs for files and directories', () => {
      expect(parseUrlOrPath('file:///C:/Users/happy/Pictures/emote.png')).toEqual({
        type: 'local',
        value: 'C:\\Users\\happy\\Pictures\\emote.png'
      });
      expect(parseUrlOrPath('file:///C:/Users/happy/Pictures/emotes/')).toEqual({
        type: 'local',
        value: 'C:\\Users\\happy\\Pictures\\emotes\\'
      });
      expect(parseUrlOrPath('file:///home/user/emotes/')).toEqual({
        type: 'local',
        value: '/home/user/emotes/'
      });
      expect(parseUrlOrPath('file:///home/user/my%20emotes/')).toEqual({
        type: 'local',
        value: '/home/user/my emotes/'
      });
      expect(parseUrlOrPath('file://localhost/home/user/emotes')).toEqual({
        type: 'local',
        value: '/home/user/emotes'
      });
    });

    it('parses direct local filesystem paths for files and directories (Windows and Linux)', () => {
      expect(parseUrlOrPath('C:\\Users\\happy\\Pictures\\emote.gif')).toEqual({
        type: 'local',
        value: 'C:\\Users\\happy\\Pictures\\emote.gif'
      });
      expect(parseUrlOrPath('D:/emotes/pack/')).toEqual({
        type: 'local',
        value: 'D:/emotes/pack/'
      });
      expect(parseUrlOrPath('/var/data/emotes')).toEqual({
        type: 'local',
        value: '/var/data/emotes'
      });
      expect(parseUrlOrPath('/home/user/Pictures/emotes/')).toEqual({
        type: 'local',
        value: '/home/user/Pictures/emotes/'
      });
      expect(parseUrlOrPath('~/Pictures/emotes')).toEqual({
        type: 'local',
        value: '~/Pictures/emotes'
      });
    });

    it('parses web URLs and normalizes protocol when omitted', () => {
      expect(parseUrlOrPath('https://example.com/emote.png')).toEqual({
        type: 'web',
        value: 'https://example.com/emote.png'
      });
      expect(parseUrlOrPath('http://example.com/dir/')).toEqual({
        type: 'web',
        value: 'http://example.com/dir/'
      });
      expect(parseUrlOrPath('cdn.discordapp.com/emojis/12345.webp')).toEqual({
        type: 'web',
        value: 'https://cdn.discordapp.com/emojis/12345.webp'
      });
    });

    it('removes surrounding quotes from pasted paths', () => {
      expect(parseUrlOrPath('"C:\\Users\\happy\\My Emotes"')).toEqual({
        type: 'local',
        value: 'C:\\Users\\happy\\My Emotes'
      });
      expect(parseUrlOrPath("'https://example.com/pic.png'")).toEqual({
        type: 'web',
        value: 'https://example.com/pic.png'
      });
    });
  });
});



