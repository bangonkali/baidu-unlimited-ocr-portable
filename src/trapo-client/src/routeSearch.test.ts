import { describe, expect, test } from 'bun:test';

import {
  validateDiagnosticsSearch,
  validateIngestSearch,
  validateModelSearch,
  validateRootSearch,
  validateSettingsSearch,
  validateWorkbenchSearch,
} from './routeSearch';

describe('route search validators', () => {
  test('keeps global downloads pane state at the root route', () => {
    expect(validateRootSearch({ downloads: 'true' })).toEqual({ downloads: true });
    expect(validateRootSearch({ downloads: '0' })).toEqual({ downloads: false });
    expect(validateRootSearch({ downloads: 'later' })).toEqual({ downloads: undefined });
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
      }),
    ).toEqual({
      file: 'abc',
      follow: false,
      labels: false,
      overlays: true,
      page: 3,
      q: 'invoice',
      region: 'r1',
    });
    expect(validateWorkbenchSearch({ page: '-2' }).page).toBeUndefined();
    expect(
      validateWorkbenchSearch({
        file_hash: 'legacy-hash',
        page_no: '2',
        region_id: 'legacy-region',
      }),
    ).toMatchObject({
      file: 'legacy-hash',
      page: 2,
      region: 'legacy-region',
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
    expect(validateIngestSearch({ model: 'm1', profile: 'p1', reprocess: 'true' })).toEqual({
      model: 'm1',
      profile: 'p1',
      reprocess: true,
    });
  });
});
