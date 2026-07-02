import { describe, expect, test } from 'bun:test';

import type { DocumentSummary, PageTextRecord } from '../../api/types';
import { visibleTextPages } from './textPreviewPages';

function page(page_no: number, text = ''): PageTextRecord {
  return { page_no, spans: [], text };
}

describe('visibleTextPages', () => {
  test('hides queued placeholder pages while a document is running', () => {
    const document: DocumentSummary = {
      display_name: 'ten-pages.pdf',
      file_hash: 'hash',
      current_page: 2,
      page_count: 10,
      processed_pages: 1,
      status: 'running',
      total_pages: 10,
    };

    expect(visibleTextPages([page(1, 'First'), page(2), page(10)], document)).toEqual([
      page(1, 'First'),
      page(2),
    ]);
  });

  test('shows all pages after terminal completion', () => {
    const document: DocumentSummary = {
      display_name: 'blank-pages.pdf',
      file_hash: 'hash',
      page_count: 2,
      processed_pages: 2,
      status: 'completed',
      total_pages: 2,
    };

    expect(visibleTextPages([page(1), page(2)], document)).toEqual([page(1), page(2)]);
  });
});
