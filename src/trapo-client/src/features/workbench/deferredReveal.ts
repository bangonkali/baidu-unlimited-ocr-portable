interface DeferredRevealOptions<TTarget extends HTMLElement> {
  findTarget: () => TTarget | null;
  reveal: (target: TTarget) => void;
  root: HTMLElement;
  timeoutMs?: number;
}

export function revealWhenAvailable<TTarget extends HTMLElement>({
  findTarget,
  reveal,
  root,
  timeoutMs = 8000,
}: DeferredRevealOptions<TTarget>) {
  let animationFrameId: number | undefined;
  let done = false;
  let observer: MutationObserver | undefined;
  let timeoutId: number | undefined;

  const cleanup = () => {
    if (done) {
      return;
    }
    done = true;
    if (animationFrameId !== undefined) {
      window.cancelAnimationFrame(animationFrameId);
    }
    if (timeoutId !== undefined) {
      window.clearTimeout(timeoutId);
    }
    observer?.disconnect();
  };

  const tryReveal = () => {
    if (done) {
      return;
    }
    const target = findTarget();
    if (!target || !root.contains(target)) {
      return;
    }
    reveal(target);
    cleanup();
  };

  tryReveal();
  if (done) {
    return cleanup;
  }

  animationFrameId = window.requestAnimationFrame(() => {
    animationFrameId = undefined;
    tryReveal();
  });
  observer = new MutationObserver(tryReveal);
  observer.observe(root, { childList: true, subtree: true });
  timeoutId = window.setTimeout(cleanup, timeoutMs);
  return cleanup;
}
