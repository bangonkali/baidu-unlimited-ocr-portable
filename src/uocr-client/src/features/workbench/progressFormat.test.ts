import { describe, expect, test } from 'bun:test';

import { documentPageLabel, percentLabel, runPageLabel } from './progressFormat';

describe('workbench progress formatting', () => {
  test('formats document and workflow page progress', () => {
    expect(
      documentPageLabel({
        current_page: 2,
        display_name: 'Sample.pdf',
        file_hash: 'hash',
        page_count: 4,
        progress_percent: 50,
        status: 'running',
        total_pages: 4,
      }),
    ).toBe('Page 2/4');
    expect(
      runPageLabel({
        current_page: 8,
        root_path: 'C:/docs',
        run_id: 'run-1',
        status: 'running',
        total_pages: 43,
      }),
    ).toBe('Page 8/43');
    expect(percentLabel(16.3)).toBe('16%');
    expect(percentLabel(120)).toBe('100%');
  });
});
