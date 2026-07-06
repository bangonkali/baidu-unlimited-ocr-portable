import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';

describe('Workbench visual style', () => {
  test('uses the VS Code activity bar left-edge active indicator', () => {
    const css = readFileSync(new URL('./WorkbenchPage.module.css', import.meta.url), 'utf8');

    expect(css).toContain('.activityLink[aria-pressed="true"]::before');
    expect(css).toContain('left: 0;');
    expect(css).toContain('width: 2px;');
    expect(css).toContain('background: var(--activity-active-border);');
    expect(css).not.toContain('border: 1px solid transparent;');
    expect(css).not.toContain('border-color: var(--border);');
  });

  test('keeps primary accents on the VS Code blue palette', () => {
    const css = readFileSync(new URL('../../styles/base.css', import.meta.url), 'utf8');

    expect(css).toContain('--activity-active-border: #0078d4;');
    expect(css).toContain('--activity-active-border: #005fb8;');
    expect(css).toContain('--accent-2: #0078d4;');
    expect(css).toContain('--accent-2: #005fb8;');
    expect(css).not.toContain('#185c37');
    expect(css).not.toContain('#6ee7a8');
    expect(css).not.toContain('#2da44e');
  });
});
