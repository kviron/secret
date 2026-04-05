import { describe, expect, it } from 'vitest';
import { en } from './locales/en';
import type { MessageKey } from './locales/en';
import { ru } from './locales/ru';
import { interpolate } from './interpolate';

describe('i18n', () => {
  it('ru contains every key from en', () => {
    const keys = Object.keys(en) as MessageKey[];
    expect(keys.length).toBeGreaterThan(0);
    for (const k of keys) {
      expect(ru[k]).toBeDefined();
      expect(typeof ru[k]).toBe('string');
    }
  });

  it('interpolate replaces placeholders', () => {
    expect(interpolate('Hello {name}', { name: 'World' })).toBe('Hello World');
    expect(interpolate('n={n}', { n: 3 })).toBe('n=3');
  });
});
