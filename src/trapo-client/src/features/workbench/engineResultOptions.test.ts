import { describe, expect, test } from 'bun:test';

import type {
  DocumentSummary,
  IngestEngineConfigRecord,
  IngestPreviewResultRecord,
  IngestRunRecord,
} from '../../api/types';
import {
  defaultRunEngineId,
  engineResultOptions,
  selectedRunEngineIdFromOptions,
} from './engineResultOptions';

describe('engineResultOptions', () => {
  test('keeps completed preview results and adds in-progress engine configs', () => {
    const options = engineResultOptions({
      document: documentFixture(),
      results: [previewResultFixture({ run_engine_id: 'engine-a' })],
      run: runFixture([
        engineConfigFixture({ run_engine_id: 'engine-a', status: 'completed' }),
        engineConfigFixture({
          engine_id: 'paddleocr-vl-1.6-gguf',
          label: 'PaddleOCR-VL 1.6',
          ordinal: 1,
          run_engine_id: 'engine-b',
          status: 'running',
        }),
        engineConfigFixture({
          engine_id: 'dots-mocr-gguf',
          label: 'dots.mocr',
          ordinal: 2,
          previewer: 'document_markdown',
          run_engine_id: 'engine-c',
          status: 'queued',
        }),
      ]),
    });

    expect(options.map((result) => [result.run_engine_id, result.status])).toEqual([
      ['engine-a', 'completed'],
      ['engine-b', 'running'],
      ['engine-c', 'queued'],
    ]);
    expect(options[1]).toMatchObject({
      engine_id: 'paddleocr-vl-1.6-gguf',
      label: 'PaddleOCR-VL 1.6',
      page_count: 13,
      previewer: 'ocr_annotation',
      runner_status: 'running',
    });
  });

  test('selects explicit result, then realtime selection, then running engine', () => {
    const results = [
      previewResultFixture({ run_engine_id: 'engine-a', status: 'completed' }),
      previewResultFixture({ run_engine_id: 'engine-b', status: 'running' }),
    ];

    expect(
      selectedRunEngineIdFromOptions({
        explicitResultId: 'engine-c',
        results,
        selectionRunEngineId: 'engine-b',
      }),
    ).toBe('engine-c');
    expect(
      selectedRunEngineIdFromOptions({
        results,
        selectionRunEngineId: 'engine-a',
      }),
    ).toBe('engine-a');
    expect(selectedRunEngineIdFromOptions({ results })).toBe('engine-b');
  });

  test('falls back to the latest ran engine and stays empty when no engine ran', () => {
    expect(
      defaultRunEngineId([
        previewResultFixture({ ordinal: 0, run_engine_id: 'engine-a', status: 'completed' }),
        previewResultFixture({ ordinal: 2, run_engine_id: 'engine-c', status: 'completed' }),
        previewResultFixture({
          ordinal: 3,
          output_count: 0,
          page_count: 0,
          run_engine_id: 'engine-d',
          status: 'queued',
        }),
      ]),
    ).toBe('engine-c');

    expect(
      defaultRunEngineId([
        previewResultFixture({
          output_count: 0,
          page_count: 0,
          run_engine_id: 'engine-a',
          status: 'queued',
        }),
      ]),
    ).toBeUndefined();
  });
});

function runFixture(engineConfigs: IngestEngineConfigRecord[]): IngestRunRecord {
  return {
    engine_configs: engineConfigs,
    root_path: 'C:\\data',
    run_id: 'run-a',
    status: 'running',
  };
}

function documentFixture(): DocumentSummary {
  return {
    display_name: 'Sample 0003.pdf',
    file_hash: 'file-a',
    page_count: 13,
    status: 'running',
  };
}

function engineConfigFixture(
  patch: Partial<IngestEngineConfigRecord> = {},
): IngestEngineConfigRecord {
  return {
    engine_id: 'tesseract-rs',
    engine_kind: 'ocr',
    label: 'Tesseract',
    ordinal: 0,
    parameters: {},
    previewer: 'ocr_annotation',
    run_engine_id: 'engine-a',
    run_id: 'run-a',
    status: 'completed',
    usable_output_count: 1,
    ...patch,
  };
}

function previewResultFixture(
  patch: Partial<IngestPreviewResultRecord> = {},
): IngestPreviewResultRecord {
  return {
    engine_id: 'tesseract-rs',
    engine_kind: 'ocr',
    label: 'Tesseract',
    ordinal: 0,
    output_count: 1,
    page_count: 13,
    previewer: 'ocr_annotation',
    provenance: {},
    run_engine_id: 'engine-a',
    run_id: 'run-a',
    runner_kind: 'native',
    runner_status: 'ready',
    status: 'completed',
    ...patch,
  };
}
