interface RectLike {
  bottom: number;
  height: number;
  left: number;
  right: number;
  top: number;
  width: number;
}

interface ScrollEndLike {
  clientHeight: number;
  scrollHeight: number;
  scrollTop: number;
}

const defaultTolerancePx = 2;

export function isRectVisibleWithinRoot(
  rootRect: RectLike,
  targetRect: RectLike,
  tolerancePx = defaultTolerancePx,
) {
  if (
    targetRect.width > rootRect.width + tolerancePx ||
    targetRect.height > rootRect.height + tolerancePx
  ) {
    return false;
  }
  return (
    targetRect.left >= rootRect.left - tolerancePx &&
    targetRect.right <= rootRect.right + tolerancePx &&
    targetRect.top >= rootRect.top - tolerancePx &&
    targetRect.bottom <= rootRect.bottom + tolerancePx
  );
}

export function needsRevealScroll(
  rootRect: RectLike,
  targetRect: RectLike,
  tolerancePx = defaultTolerancePx,
) {
  return !isRectVisibleWithinRoot(rootRect, targetRect, tolerancePx);
}

export function isScrolledToBottom(root: ScrollEndLike, tolerancePx = defaultTolerancePx) {
  return root.scrollHeight - root.scrollTop - root.clientHeight <= tolerancePx;
}
