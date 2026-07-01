import { describe, expect, test } from 'bun:test';

import type { DocumentSummary, PageTextRecord } from '../../api/types';
import { fileMarkdown, filePlainText, isPageTextComplete, markdownToPlainText } from './textExport';

const pageOne: PageTextRecord = {
  page_no: 1,
  spans: [],
  text: '# Invoice\n\n[Supplier](https://example.test)\n\n**Total:** `1,240.00`',
};

const pageTwo: PageTextRecord = {
  page_no: 2,
  spans: [],
  text: 'Second page',
};

describe('text export helpers', () => {
  test('infers completed page status from document progress', () => {
    const running: DocumentSummary = {
      display_name: 'sample.pdf',
      file_hash: 'hash',
      current_page: 2,
      page_count: 3,
      processed_pages: 1,
      status: 'running',
    };
    expect(isPageTextComplete(pageOne, running)).toBe(true);
    expect(isPageTextComplete(pageTwo, running)).toBe(false);
    expect(isPageTextComplete(pageTwo, { ...running, status: 'completed' })).toBe(true);
  });

  test('copies multi-page markdown and plain text with page boundaries', () => {
    expect(fileMarkdown([pageOne, pageTwo])).toContain('## Page 1');
    expect(fileMarkdown([pageOne, pageTwo])).toContain('## Page 2');
    expect(filePlainText([pageOne, pageTwo])).toContain('-- Page 1 --');
    expect(markdownToPlainText(pageOne.text)).toBe('Invoice\n\nSupplier\n\nTotal: 1,240.00');
  });
});
