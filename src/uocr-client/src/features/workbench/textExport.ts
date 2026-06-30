import type { DocumentSummary, PageTextRecord } from '../../api/types';

const COMPLETED_STATUSES = new Set(['completed', 'completed_with_errors']);

export function isPageTextComplete(page: PageTextRecord, document?: DocumentSummary) {
  if (!document) {
    return false;
  }
  if (COMPLETED_STATUSES.has(document.status)) {
    return true;
  }
  if ((document.processed_pages ?? 0) >= page.page_no) {
    return true;
  }
  return document.status === 'running' && (document.current_page ?? 1) > page.page_no;
}

export function pageMarkdown(page: PageTextRecord) {
  return page.text.trim();
}

export function pagePlainText(page: PageTextRecord) {
  return markdownToPlainText(pageMarkdown(page));
}

export function fileMarkdown(pages: PageTextRecord[]) {
  if (pages.length <= 1) {
    return pageMarkdown(pages[0] ?? emptyPage());
  }
  return pages
    .map((page) => `## Page ${page.page_no}\n\n${pageMarkdown(page)}`)
    .filter((value) => value.trim().length > 0)
    .join('\n\n');
}

export function filePlainText(pages: PageTextRecord[]) {
  if (pages.length <= 1) {
    return pagePlainText(pages[0] ?? emptyPage());
  }
  return pages
    .map((page) => `-- Page ${page.page_no} --\n\n${pagePlainText(page)}`)
    .filter((value) => value.trim().length > 0)
    .join('\n\n');
}

export function markdownToPlainText(markdown: string) {
  return markdown
    .replace(/```[\s\S]*?```/g, (block) => block.replace(/```[^\n]*\n?|```/g, ''))
    .replace(/!\[([^\]]*)\]\([^)]+\)/g, '$1')
    .replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
    .replace(/(^|\n)\s{0,3}#{1,6}\s+/g, '$1')
    .replace(/(^|\n)\s{0,3}>\s?/g, '$1')
    .replace(/(^|\n)\s*[-*+]\s+/g, '$1')
    .replace(/(^|\n)\s*\d+\.\s+/g, '$1')
    .replace(/[*_~`]+/g, '')
    .replace(/[ \t]+\n/g, '\n')
    .trim();
}

function emptyPage(): PageTextRecord {
  return { page_no: 1, spans: [], text: '' };
}
