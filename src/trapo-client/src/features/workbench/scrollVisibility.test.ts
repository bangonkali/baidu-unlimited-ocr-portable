import { describe, expect, test } from 'bun:test';

import { isRectVisibleWithinRoot, isScrolledToBottom, needsRevealScroll } from './scrollVisibility';

const root = {
  bottom: 500,
  height: 400,
  left: 50,
  right: 650,
  top: 100,
  width: 600,
};

describe('scroll visibility helpers', () => {
  test('does not reveal fully visible targets', () => {
    const target = {
      bottom: 320,
      height: 120,
      left: 120,
      right: 420,
      top: 200,
      width: 300,
    };

    expect(isRectVisibleWithinRoot(root, target)).toBe(true);
    expect(needsRevealScroll(root, target)).toBe(false);
  });

  test('reveals targets outside each root edge', () => {
    expect(needsRevealScroll(root, { ...root, bottom: 190, top: 90 })).toBe(true);
    expect(needsRevealScroll(root, { ...root, bottom: 510, top: 410 })).toBe(true);
    expect(needsRevealScroll(root, { ...root, left: 40, right: 140 })).toBe(true);
    expect(needsRevealScroll(root, { ...root, left: 560, right: 660 })).toBe(true);
  });

  test('allows small subpixel boundary drift', () => {
    expect(
      isRectVisibleWithinRoot(root, {
        bottom: 501.5,
        height: 401,
        left: 48.5,
        right: 651.5,
        top: 98.5,
        width: 601,
      }),
    ).toBe(true);
  });

  test('reveals targets larger than the scroll root', () => {
    expect(
      needsRevealScroll(root, {
        bottom: 490,
        height: 420,
        left: 70,
        right: 630,
        top: 120,
        width: 560,
      }),
    ).toBe(true);
  });

  test('detects when a scroll root is already at the bottom', () => {
    expect(isScrolledToBottom({ clientHeight: 400, scrollHeight: 1000, scrollTop: 599 })).toBe(
      true,
    );
    expect(isScrolledToBottom({ clientHeight: 400, scrollHeight: 1000, scrollTop: 580 })).toBe(
      false,
    );
  });
});
