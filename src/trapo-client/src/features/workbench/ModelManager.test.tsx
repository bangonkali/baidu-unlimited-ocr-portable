import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import { fixtureDownloadingModels, fixtureModels } from '../../stories/fixtures/workbenchFixtures';
import { activeDownloadItems, DownloadManager } from './DownloadManager';
import { ModelDetailPanel } from './ModelDetailPanel';
import { ModelManager } from './ModelManager';
import { formatBytes, formatEta, formatPercent, formatRate } from './modelDownloadFormat';
import { visibleModels } from './modelLibrary';

describe('model download formatting', () => {
  test('formats progress values for the model dashboard', () => {
    expect(formatBytes(4_900_000_000)).toBe('4.6 GiB');
    expect(formatRate(10 * 1024 * 1024)).toBe('10.00 MiB/s');
    expect(formatEta(125)).toBe('2m 5s');
    expect(formatPercent(36.66)).toBe('36.7%');
  });
});

describe('ModelManager', () => {
  test('renders downloaded model state', () => {
    const html = renderToString(
      <ModelManager
        models={fixtureModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
        routeSearch={{ view: 'cards' }}
      />,
    );
    expect(html).toContain('Authenticated with HF_TOKEN');
    expect(html).toContain('Re-download');
    expect(html).toContain('Unlimited-OCR IQ2_M');
    expect(html).toContain('In Use');
    expect(html).toContain('Recommended');
  });

  test('renders live download state', () => {
    const html = renderToString(
      <ModelManager
        models={fixtureDownloadingModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
        routeSearch={{ view: 'cards' }}
      />,
    );
    expect(html).toContain('Cancel');
    expect(html).toContain('downloading');
  });

  test('renders active file download details in the download manager', () => {
    const html = renderToString(
      <DownloadManager
        models={fixtureDownloadingModels.models}
        onCancelModel={() => undefined}
        onClose={() => undefined}
      />,
    );
    expect(html).toContain('active files');
    expect(html).toContain('45.0%');
    expect(html).toContain('11.25 MiB/s');
    expect(html).toContain('Unlimited-OCR-Q4_K_M.gguf');
    expect(activeDownloadItems(fixtureModels.models)).toHaveLength(0);
  });

  test('renders compact grid headers by default', () => {
    const html = renderToString(
      <ModelManager
        models={fixtureModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
      />,
    );
    expect(html).toContain('VRAM / Tier');
    expect(html).toContain('Downloads');
  });

  test('filters active downloads and sorts by size', () => {
    const downloads = visibleModels(fixtureDownloadingModels.models, {
      scope: 'downloads',
      status: 'all',
    });
    expect(downloads.map((model) => model.model_id)).toEqual(['unlimited-ocr-q4-k-m']);

    const bySize = visibleModels(fixtureModels.models, { dir: 'asc', sort: 'size' });
    expect(bySize[0]?.model_id).toBe('unlimited-ocr-iq2-m');
  });

  test('renders dedicated model detail surface', () => {
    const html = renderToString(
      <ModelDetailPanel
        model={fixtureModels.models[0]}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onSelectModel={() => undefined}
      />,
    );
    expect(html).toContain('Model Metadata');
    expect(html).toContain('sahilchachra/Unlimited-OCR-GGUF');
    expect(html).toContain('Download Progress');
  });
});
