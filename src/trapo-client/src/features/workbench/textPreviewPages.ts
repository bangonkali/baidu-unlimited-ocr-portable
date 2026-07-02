import type { DocumentSummary, PageTextRecord } from '../../api/types';

const TERMINAL_STATUSES = new Set(['completed', 'completed_with_errors']);

export function visibleTextPages(pages: PageTextRecord[], document?: DocumentSummary) {
  return pages.filter((page) => isPageTextStarted(page, document));
}

export function isPageTextStarted(page: PageTextRecord, document?: DocumentSummary) {
  if (page.text.trim().length > 0 || page.spans.length > 0) {
    return true;
  }
  if (!document) {
    return false;
  }
  if (TERMINAL_STATUSES.has(document.status)) {
    return true;
  }
  const processedPages = document.processed_pages ?? 0;
  if (page.page_no <= processedPages) {
    return true;
  }
  if (document.current_page !== undefined && document.current_page !== null) {
    return page.page_no <= document.current_page;
  }
  return document.status === 'running' && page.page_no <= processedPages + 1;
}
