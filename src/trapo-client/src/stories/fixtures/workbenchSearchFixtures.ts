import type {
  HybridSearchFileResult,
  HybridSearchHit,
  UsedEmbeddingModelRecord,
} from '../../api/types';

export const fixtureUsedEmbeddingModels: UsedEmbeddingModelRecord[] = [
  {
    dimension: 768,
    display_name: 'Nomic Embed Text',
    model_id: 'nomic-embed-text-v1-5-q4-k-m',
    provider: 'Nomic',
  },
];

const firstHit: HybridSearchHit = {
  annotation_id: '01902c7e-0000-7000-8000-000000000101',
  category: 'page_text',
  file_hash: 'hash-invoice-014',
  hit_source: 'fts+vss',
  model_id: 'nomic-embed-text-v1-5-q4-k-m',
  page_no: 1,
  rank: 1,
  relevance_score: 0.032,
  score: 12,
  segment_id: '01902c7e-0000-7000-8000-000000000001',
  text: 'Supplier asuka invoice total: 1,240.00',
};

const secondHit: HybridSearchHit = {
  annotation_id: '01902c7e-0000-7000-8000-000000000102',
  category: 'page_text',
  file_hash: 'hash-shipping-form',
  hit_source: 'fts',
  model_id: null,
  page_no: 2,
  rank: 2,
  relevance_score: 0.016,
  score: 8,
  segment_id: '01902c7e-0000-7000-8000-000000000002',
  text: 'Shipping form notes reference asuka in the recipient memo.',
};

export const fixtureSearchHits: HybridSearchHit[] = [firstHit, secondHit];

export const fixtureSearchFiles: HybridSearchFileResult[] = [
  {
    file_hash: 'hash-invoice-014',
    hit_count: 1,
    relevance_score: firstHit.relevance_score,
    hits: [firstHit],
  },
  {
    file_hash: 'hash-shipping-form',
    hit_count: 1,
    relevance_score: secondHit.relevance_score,
    hits: [secondHit],
  },
];
