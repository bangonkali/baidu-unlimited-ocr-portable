import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { renderToString } from 'react-dom/server';

import { ServiceOfflinePage } from './ServiceOfflinePage';
import { StatusBar } from './StatusBar';

describe('StatusBar shutdown controls', () => {
  test('renders the always-visible shutdown button', () => {
    const html = renderToString(
      <StatusBar
        documentCount={2}
        downloadsActiveCount={0}
        downloadsOpen={false}
        host="127.0.0.1:8765"
        onDownloadsToggle={() => undefined}
        onShutdown={() => undefined}
        realtimeState="connected"
        runState="idle"
        runtime="windows-x64 / cuda"
        selectedRoot="C:\\data"
      />,
    );

    expect(html).toContain('aria-label="Shut down Trapo"');
    expect(html).toContain('aria-label="Notifications"');
    expect(html).toContain('Downloads');
    expect(html).toContain('127.0.0.1:8765');
    expect(html.indexOf('aria-label="Notifications"')).toBeGreaterThan(
      html.indexOf('127.0.0.1:8765'),
    );
  });

  test('uses explicit status bar contrast tokens for light theme', () => {
    const css = readFileSync(new URL('../../styles/base.css', import.meta.url), 'utf8');

    expect(css).toContain('--status-foreground: #ffffff');
    expect(css).toContain('--status-border');
    expect(css).toContain('--status-hover');
  });

  test('renders the offline page copy', () => {
    const html = renderToString(
      <ServiceOfflinePage
        message="The local server accepted the shutdown request."
        mode="shutting_down"
        onRetry={() => undefined}
      />,
    );

    expect(html).toContain('Trapo is shutting down');
    expect(html).toContain('Retry');
    expect(html).toContain('Restart trapo-server');
  });
});
