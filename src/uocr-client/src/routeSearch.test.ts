import { describe, expect, test } from 'bun:test';

import {
  validateDiagnosticsSearch,
  validateModelSearch,
  validateSettingsSearch,
  validateWorkbenchSearch,
} from './routeSearch';

describe('route search validators', () => {
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
    expect(validateSettingsSearch({ section: 'runtime' })).toEqual({ section: 'runtime' });
    expect(validateDiagnosticsSearch({ q: 'cuda', run: 'run-1', tab: 'logs' })).toEqual({
      q: 'cuda',
      run: 'run-1',
      tab: 'logs',
    });
  });
});
