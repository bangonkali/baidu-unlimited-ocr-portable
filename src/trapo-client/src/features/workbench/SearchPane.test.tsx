import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { renderToString } from 'react-dom/server';

import type { DocumentSummary, HybridSearchFileResult, HybridSearchHit } from '../../api/types';
import { SearchPane } from './SearchPane';

describe('SearchPane', () => {
  test('renders search results as expanded file tree nodes by default', () => {
    const html = renderSearchPane('tree');

    expect(html).toContain('role="tree"');
    expect(html).toContain('aria-label="Collapse asuka.pdf"');
    expect(html).toContain('aria-expanded="true"');
    expect(html).toContain('fts+vss');
    expect(html).toContain('<mark>asuka</mark>');
  });

  test('renders backend ranked hits in flat ranked view', () => {
    const html = renderSearchPane('ranked').replaceAll('<!-- -->', '');

    expect(html).toContain('#1');
    expect(html).toContain('asuka.pdf');
    expect(html.indexOf('#1')).toBeLessThan(html.indexOf('#2'));
  });

  test('keeps fts and vss hits on neutral row backgrounds', () => {
    const css = readFileSync(new URL('./SearchView.module.css', import.meta.url), 'utf8');

    expect(css).not.toContain('data-source');
    expect(css).not.toContain('var(--success)');
  });
});

function renderSearchPane(view: 'tree' | 'ranked') {
  return renderToString(
    <SearchPane
      documents={new Map(documents.map((document) => [document.file_hash, document]))}
      files={files}
      hits={hits}
      loading={false}
      models={[
        {
          dimension: 768,
          display_name: 'Nomic Embed Text',
          model_id: 'nomic-embed-text-v1-5-q4-k-m',
          provider: 'Nomic',
        },
      ]}
      query="asuka"
      runs={[]}
      selectedModelId="nomic-embed-text-v1-5-q4-k-m"
      view={view}
      onChange={() => undefined}
      onSelectHit={() => undefined}
    />,
  );
}

const documents: DocumentSummary[] = [
  {
    display_name: 'asuka.pdf',
    file_hash: 'hash-asuka',
    page_count: 2,
    status: 'completed',
  },
  {
    display_name: 'eva-notes.pdf',
    file_hash: 'hash-notes',
    page_count: 1,
    status: 'completed',
  },
];

const firstHit = hit({
  file_hash: 'hash-asuka',
  hit_source: 'fts+vss',
  rank: 1,
  relevance_score: 0.03,
  segment_id: '01902c7e-0000-7000-8000-000000000001',
  text: 'asuka search result from multiple sources',
});

const secondHit = hit({
  file_hash: 'hash-notes',
  hit_source: 'fts',
  rank: 2,
  relevance_score: 0.01,
  segment_id: '01902c7e-0000-7000-8000-000000000002',
  text: 'asuka appears in the notes',
});

const hits: HybridSearchHit[] = [firstHit, secondHit];

const files: HybridSearchFileResult[] = [
  {
    file_hash: 'hash-asuka',
    hit_count: 1,
    relevance_score: 0.03,
    hits: [firstHit],
  },
  {
    file_hash: 'hash-notes',
    hit_count: 1,
    relevance_score: 0.01,
    hits: [secondHit],
  },
];

function hit(overrides: Partial<HybridSearchHit>): HybridSearchHit {
  return {
    annotation_id: '01902c7e-0000-7000-8000-000000000010',
    category: 'page_text',
    file_hash: 'hash-asuka',
    hit_source: 'fts',
    model_id: null,
    page_no: 1,
    rank: 1,
    relevance_score: 0.01,
    score: 1,
    segment_id: '01902c7e-0000-7000-8000-000000000099',
    text: 'asuka',
    ...overrides,
  };
}
