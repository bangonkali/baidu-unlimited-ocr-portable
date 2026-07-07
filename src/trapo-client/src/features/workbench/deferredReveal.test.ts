import { afterEach, describe, expect, test } from 'bun:test';

import { revealWhenAvailable } from './deferredReveal';

const originalMutationObserver = globalThis.MutationObserver;
const originalWindow = globalThis.window;

afterEach(() => {
  defineGlobal('MutationObserver', originalMutationObserver);
  defineGlobal('window', originalWindow);
  FakeMutationObserver.current = undefined;
});

describe('revealWhenAvailable', () => {
  test('reveals immediately when the target is already mounted', () => {
    const target = {} as HTMLElement;
    const root = fakeRoot(() => target);
    let reveals = 0;

    const cleanup = revealWhenAvailable({
      findTarget: () => target,
      reveal: () => {
        reveals += 1;
      },
      root,
    });

    expect(reveals).toBe(1);
    cleanup();
    expect(reveals).toBe(1);
  });

  test('waits for a later DOM mutation when the target is not mounted yet', () => {
    installDeferredRevealDom();
    let target: HTMLElement | null = null;
    const root = fakeRoot(() => target);
    let reveals = 0;

    const cleanup = revealWhenAvailable({
      findTarget: () => target,
      reveal: () => {
        reveals += 1;
      },
      root,
    });

    expect(reveals).toBe(0);
    expect(FakeMutationObserver.current?.observed).toBe(true);

    target = {} as HTMLElement;
    FakeMutationObserver.current?.trigger();

    expect(reveals).toBe(1);
    expect(FakeMutationObserver.current?.disconnected).toBe(true);
    cleanup();
  });
});

class FakeMutationObserver {
  static current: FakeMutationObserver | undefined;
  disconnected = false;
  observed = false;
  private readonly callback: MutationCallback;

  constructor(callback: MutationCallback) {
    this.callback = callback;
    FakeMutationObserver.current = this;
  }

  disconnect() {
    this.disconnected = true;
  }

  observe() {
    this.observed = true;
  }

  trigger() {
    this.callback([], this as unknown as MutationObserver);
  }
}

function fakeRoot(target: () => HTMLElement | null) {
  return {
    contains: (node: Node | null) => node === target(),
  } as unknown as HTMLElement;
}

function installDeferredRevealDom() {
  const fakeWindow = {
    cancelAnimationFrame: () => undefined,
    clearTimeout: () => undefined,
    requestAnimationFrame: () => 1,
    setTimeout: () => 1,
  };
  defineGlobal('MutationObserver', FakeMutationObserver as unknown as typeof MutationObserver);
  defineGlobal('window', fakeWindow as unknown as Window & typeof globalThis);
}

function defineGlobal<TName extends keyof typeof globalThis>(
  name: TName,
  value: (typeof globalThis)[TName],
) {
  Object.defineProperty(globalThis, name, { configurable: true, value });
}
