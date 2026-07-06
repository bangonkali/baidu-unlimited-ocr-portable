import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import type {
  DiagnosticPipelineTaskRecord,
  DocumentSummary,
  IngestRunRecord,
} from '../../api/types';
import { buildDocumentTree } from './ExplorerTree';

describe('buildDocumentTree', () => {
  test('orders page children by numeric page number', () => {
    const tree = buildDocumentTree({
      documents: [documentSummary({ page_count: 12 })],
      fallbackRootPath: 'C:\\incoming',
      onSelectDocument: () => undefined,
      runs: [],
      scope: 'run',
    });

    const documentNode = tree.nodes[0]?.children[0];
    expect(documentNode?.children?.map((node) => node.label)).toEqual([
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

  test('filters default tree to the selected run membership', () => {
    const tree = buildDocumentTree({
      documents: [
        documentSummary({ file_hash: 'hash-old', relative_path: 'old.pdf' }),
        documentSummary({ file_hash: 'hash-latest', relative_path: 'latest.pdf' }),
      ],
      onSelectDocument: () => undefined,
      runId: 'run-latest',
      runs: [
        ingestRun({ file_hashes: ['hash-latest'], run_id: 'run-latest' }),
        ingestRun({ file_hashes: ['hash-old'], run_id: 'run-old' }),
      ],
      scope: 'run',
    });

    expect(tree.documentCount).toBe(1);
    expect(tree.nodes[0]?.children.map((node) => node.label)).toEqual(['latest.pdf']);
  });

  test('keeps all-runs roots separate even when root folder names match', () => {
    const tree = buildDocumentTree({
      documents: [
        documentSummary({ file_hash: 'hash-a', relative_path: 'forms/a.pdf' }),
        documentSummary({ file_hash: 'hash-b', relative_path: 'forms/b.pdf' }),
      ],
      onSelectDocument: () => undefined,
      runs: [
        ingestRun({
          file_hashes: ['hash-a'],
          root_path: 'C:\\work\\incoming',
          run_id: 'run-a',
        }),
        ingestRun({
          file_hashes: ['hash-b'],
          root_path: 'D:\\archive\\incoming',
          run_id: 'run-b',
        }),
      ],
      scope: 'all',
    });

    expect(tree.nodes.map((node) => node.id)).toEqual(['root:run-a', 'root:run-b']);
    expect(tree.nodes.map((node) => node.label)).toEqual([
      'C:\\work\\incoming',
      'D:\\archive\\incoming',
    ]);
    expect(tree.nodes[0]?.children[0]?.children?.map((node) => node.label)).toEqual(['a.pdf']);
    expect(tree.nodes[1]?.children[0]?.children?.map((node) => node.label)).toEqual(['b.pdf']);
  });

  test('passes the owning run id when a document is selected', () => {
    const selections: Array<[string, number | undefined, string | undefined]> = [];
    const tree = buildDocumentTree({
      documents: [documentSummary({ file_hash: 'hash-a', relative_path: 'a.pdf' })],
      onSelectDocument: (fileHash, pageNo, runId) => {
        selections.push([fileHash, pageNo, runId]);
      },
      runs: [ingestRun({ file_hashes: ['hash-a'], run_id: 'run-a' })],
      scope: 'all',
    });

    tree.nodes[0]?.children[0]?.onSelect?.();
    expect(selections).toEqual([['hash-a', 1, 'run-a']]);
  });

  test('marks documents and pages while a rag pipeline task is active', () => {
    const tree = buildDocumentTree({
      documents: [documentSummary({ page_count: 2 })],
      onSelectDocument: () => undefined,
      pipelineTasks: [pipelineTask()],
      runId: 'run-a',
      runs: [ingestRun({ file_hashes: ['hash-long-document'], run_id: 'run-a' })],
      scope: 'run',
    });

    const documentNode = tree.nodes[0]?.children[0];
    const firstPageNode = documentNode?.children?.[0];
    const html = renderToString(
      <>
        {documentNode?.badge}
        {firstPageNode?.badge}
      </>,
    );

    expect(html.match(/data-task-kind="text_index"/g)).toHaveLength(2);
    expect(html).toContain('data-task-status="running"');
  });
});

function documentSummary(overrides: Partial<DocumentSummary> = {}): DocumentSummary {
  return {
    current_page: 1,
    display_name: 'long-document.pdf',
    error: undefined,
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

function ingestRun(overrides: Partial<IngestRunRecord> = {}): IngestRunRecord {
  return {
    file_hashes: [],
    root_path: 'C:\\incoming',
    run_id: 'run-a',
    status: 'completed',
    ...overrides,
  };
}

function pipelineTask(
  overrides: Partial<DiagnosticPipelineTaskRecord> = {},
): DiagnosticPipelineTaskRecord {
  return {
    error: null,
    finished_at: null,
    origin_run_id: 'run-a',
    params: { source_run_id: 'run-a' },
    queued_at: '2026-07-07T00:00:00Z',
    result: {},
    runner_id: 'local-runner-1',
    started_at: '2026-07-07T00:00:01Z',
    status: 'running',
    task_id: 'task-a',
    task_kind: 'text_index',
    ...overrides,
  };
}
