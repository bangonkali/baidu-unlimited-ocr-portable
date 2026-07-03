import { describe, expect, test } from 'bun:test';

import type { DocumentSummary } from '../../api/types';
import { buildDocumentTree } from './ExplorerTree';

describe('buildDocumentTree', () => {
  test('orders page children by numeric page number', () => {
    const tree = buildDocumentTree(
      [documentSummary({ page_count: 12 })],
      () => undefined,
      undefined,
      'C:\\incoming',
    );

    const documentNode = tree.nodes[0]?.children[0];
    expect(documentNode?.children.map((node) => node.label)).toEqual([
      'Page 1',
      'Page 2',
      'Page 3',
      'Page 4',
      'Page 5',
      'Page 6',
      'Page 7',
      'Page 8',
      'Page 9',
      'Page 10',
      'Page 11',
      'Page 12',
    ]);
  });
});

function documentSummary(overrides: Partial<DocumentSummary> = {}): DocumentSummary {
  return {
    current_page: 1,
    display_name: 'long-document.pdf',
    error: null,
    file_hash: 'hash-long-document',
    page_count: 1,
    processed_pages: 0,
    progress_percent: 0,
    regions: 0,
    relative_path: 'long-document.pdf',
    status: 'queued',
    total_pages: 1,
    ...overrides,
  };
}
