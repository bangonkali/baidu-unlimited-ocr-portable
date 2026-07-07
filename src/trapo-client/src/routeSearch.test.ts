import { describe, expect, test } from 'bun:test';

import {
  validateDiagnosticsSearch,
  validateIngestSearch,
  validateModelSearch,
  validateRootSearch,
  validateSearchSearch,
  validateSettingsSearch,
  validateWorkbenchSearch,
  withDownloadsPaneSearch,
} from './routeSearch';

describe('route search validators', () => {
  test('keeps global downloads pane state at the root route', () => {
    expect(validateRootSearch({ downloads: 'true' })).toEqual({ downloads: true });
    expect(validateRootSearch({ downloads: '0' })).toEqual({ downloads: false });
    expect(validateRootSearch({ downloads: 'later' })).toEqual({ downloads: undefined });
  });

  test('closes the downloads pane without dropping route-local search state', () => {
    expect(withDownloadsPaneSearch({ downloads: true, section: 'models' }, false)).toEqual({
      downloads: undefined,
      section: 'models',
    });
    expect(withDownloadsPaneSearch({ q: 'invoice' }, true)).toEqual({
      downloads: true,
      q: 'invoice',
    });
  });

  test('keeps shareable workbench state and drops invalid values', () => {
    expect(
      validateWorkbenchSearch({
        file: 'abc',
        follow: '0',
        labels: 'false',
        overlays: '1',
        page: '3',
        q: 'invoice',
        region: 'r1',
        run: 'run-a',
        run_scope: 'all',
      }),
    ).toEqual({
      file: 'abc',
      follow: false,
      labels: false,
      overlays: true,
      page: 3,
      q: 'invoice',
      region: 'r1',
      run: 'run-a',
      run_scope: 'all',
    });
    expect(validateWorkbenchSearch({ page: '-2' }).page).toBeUndefined();
    expect(validateWorkbenchSearch({ run_scope: 'current' }).run_scope).toBeUndefined();
    expect(
      validateWorkbenchSearch({
        file_hash: 'legacy-hash',
        page_no: '2',
        region_id: 'legacy-region',
        runScope: 'all',
        run_id: 'legacy-run',
      }),
    ).toMatchObject({
      file: 'legacy-hash',
      page: 2,
      region: 'legacy-region',
      run: 'legacy-run',
      run_scope: 'all',
    });
  });

  test('validates model, settings, and diagnostics route state', () => {
    expect(
      validateModelSearch({ dir: 'desc', sort: 'size', status: 'active', view: 'grid' }),
    ).toEqual({
      dir: 'desc',
      model: undefined,
      sort: 'size',
      status: 'active',
      view: 'grid',
    });
    expect(validateSettingsSearch({ section: 'appearance' })).toEqual({ section: 'appearance' });
    expect(
      validateDiagnosticsSearch({
        component: 'models',
        level: 'INFO',
        q: 'cuda',
        run: 'run-1',
        status: 'running',
        tab: 'logs',
      }),
    ).toEqual({
      component: 'models',
      level: 'INFO',
      q: 'cuda',
      run: 'run-1',
      status: 'running',
      tab: 'logs',
    });
    expect(
      validateIngestSearch({
        engine_id: 'pdfium-unlimited-ocr',
        model: 'm1',
        profile: 'p1',
        reprocess: 'true',
        restart_run: 'run-1',
        root_path: '/data/incoming',
        runtime_id: 'cuda',
      }),
    ).toEqual({
      engine: 'pdfium-unlimited-ocr',
      model: 'm1',
      profile: 'p1',
      reprocess: true,
      restart: 'run-1',
      root: '/data/incoming',
      runtime: 'cuda',
    });
  });

  test('validates search route presentation state', () => {
    expect(
      validateSearchSearch({
        embedding_model: 'nomic-embed-text-v1-5-q4-k-m',
        q: 'asuka',
        run_id: 'run-a',
        view: 'ranked',
      }),
    ).toEqual({
      model: 'nomic-embed-text-v1-5-q4-k-m',
      q: 'asuka',
      run: 'run-a',
      view: 'ranked',
    });
    expect(validateSearchSearch({ view: 'cards' }).view).toBeUndefined();
  });
});
