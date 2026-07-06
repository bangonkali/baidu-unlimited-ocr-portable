import { describe, expect, test } from 'bun:test';

import {
  fixtureDownloadedEmbeddingModel,
  fixtureModels,
} from '../../stories/fixtures/workbenchFixtures';
import { recommendedEmbeddingModel, recommendedOcrModel } from './ingestWizardModels';

describe('ingest wizard model recommendations', () => {
  test('recommends the selected downloaded OCR model', () => {
    expect(recommendedOcrModel(fixtureModels.models)?.model_id).toBe('unlimited-ocr-q4-k-m');
  });

  test('recommends Nomic as the first embedding model when none is downloaded', () => {
    expect(recommendedEmbeddingModel(fixtureModels.models)?.model_id).toBe(
      'nomic-embed-text-v1-5-q4-k-m',
    );
  });

  test('prefers a downloaded Nomic embedding model', () => {
    expect(
      recommendedEmbeddingModel([
        ...fixtureModels.models.filter((model) => model.model_kind !== 'embedding'),
        fixtureDownloadedEmbeddingModel,
      ])?.status,
    ).toBe('downloaded');
  });
});
