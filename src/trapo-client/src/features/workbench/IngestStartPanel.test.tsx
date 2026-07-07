import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import {
  fixtureDownloadedEmbeddingModel,
  fixtureModels,
  fixtureRuns,
} from '../../stories/fixtures/workbenchFixtures';
import { IngestStartPanel, isIngestBusy } from './IngestStartPanel';
import {
  defaultEnginePlan,
  enginePlanFromPresetIds,
  enginePlanFromRunConfigs,
} from './ingestEnginePlan';
import { buildIngestWizardStartOptions } from './ingestWizardStart';

describe('IngestStartPanel', () => {
  test('locks start when a run is active', () => {
    expect(
      isIngestBusy(
        {
          active_run_id: fixtureRuns[0]?.run_id,
          default_profile: 'experimental-exact-prefill-q4',
          state: 'running',
          supported_inputs: [],
        },
        fixtureRuns[0],
      ),
    ).toBe(true);
  });

  test('renders folder, model, profile, and run status controls', () => {
    const html = renderToString(
      <IngestStartPanel
        activeRun={fixtureRuns[0]}
        enginePresets={[]}
        model={fixtureModels.models[0]}
        models={fixtureModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onModelChange={() => undefined}
        onPickFolder={() => undefined}
        onGenerateEmbedding={() => undefined}
        onProfileChange={() => undefined}
        onRootPathChange={() => undefined}
        onStart={() => undefined}
        onStartTextIndex={() => undefined}
        onStop={() => undefined}
        profiles={fixtureModels.profiles}
        rootPath="C:\\data\\incoming"
        runs={fixtureRuns}
        selectedProfile="experimental-exact-prefill-q4"
        status={{
          active_run_id: fixtureRuns[0]?.run_id,
          default_profile: 'experimental-exact-prefill-q4',
          state: 'running',
          supported_inputs: [],
        }}
      />,
    );
    expect(html).toContain('Start Ingest');
    expect(html).toContain('Ready Check');
    expect(html).toContain('Pipeline');
    expect(html).toContain('Choose Folder');
    expect(html).toContain('Unlimited-OCR Q4_K_M');
    expect(html).toContain('Nomic Embed Text v1.5 Q4_K_M');
    expect(html).toContain('Download required model');
    expect(html).toContain('Stop Active Run');
    expect(html).toContain('disabled=""');
  });

  test('renders folder picker fallback errors', () => {
    const html = renderToString(
      <IngestStartPanel
        enginePresets={[]}
        folderDialogError="native Linux folder dialog requires zenity or kdialog. Paste a folder path manually."
        model={fixtureModels.models[0]}
        models={fixtureModels}
        onCancelModel={() => undefined}
        onDownloadModel={() => undefined}
        onModelChange={() => undefined}
        onPickFolder={() => undefined}
        onGenerateEmbedding={() => undefined}
        onProfileChange={() => undefined}
        onRootPathChange={() => undefined}
        onStart={() => undefined}
        onStartTextIndex={() => undefined}
        onStop={() => undefined}
        profiles={fixtureModels.profiles}
        rootPath=""
        runs={fixtureRuns}
        selectedProfile="experimental-exact-prefill-q4"
      />,
    );
    expect(html).toContain('role="alert"');
    expect(html).toContain('Paste a folder path manually.');
  });
});

describe('ingest engine plan options', () => {
  test('builds start options with embedding model details when enabled', () => {
    expect(
      buildIngestWizardStartOptions({
        embeddingAfterIngest: true,
        reprocess: false,
        selectedEmbeddingModel: fixtureDownloadedEmbeddingModel,
        selectedEmbeddingModelId: fixtureDownloadedEmbeddingModel.model_id,
        textIndexAfterIngest: true,
      }),
    ).toEqual({
      embeddingAfterIngest: true,
      embeddingDimension: 768,
      embeddingModelId: 'nomic-embed-text-v1-5-q4-k-m',
      reprocess: false,
      textIndexAfterIngest: true,
    });
  });

  test('builds start options with ordered engine selections', () => {
    const enginePlan = defaultEnginePlan(fixtureEnginePresets, 'experimental-exact-prefill-q4');
    expect(
      buildIngestWizardStartOptions({
        embeddingAfterIngest: false,
        enginePlan,
        enginePresets: fixtureEnginePresets,
        reprocess: true,
        selectedEmbeddingModelId: '',
        textIndexAfterIngest: false,
      }),
    ).toMatchObject({
      engines: [
        {
          engine_id: 'unlimited-ocr-ffi',
          engine_kind: 'ocr',
          ordinal: 0,
          preset_id: 'ocr-unlimited-ocr-ffi',
          profile_id: 'experimental-exact-prefill-q4',
        },
      ],
      reprocess: true,
    });
  });

  test('hydrates engine plans from query ids and persisted run configs', () => {
    expect(
      enginePlanFromPresetIds(
        ['du-dots-mocr-gguf', 'ocr-unlimited-ocr-ffi'],
        fixtureEnginePresets,
        'experimental-exact-prefill-q4',
      ).map((item) => item.engineId),
    ).toEqual(['dots-mocr-gguf', 'unlimited-ocr-ffi']);

    const fromRun = enginePlanFromRunConfigs(
      [
        {
          engine_id: 'dots-mocr-gguf',
          engine_kind: 'document_understanding',
          label: 'dots.mOCR',
          ordinal: 1,
          parameters: { mode: 'markdown' },
          previewer: 'document_markdown',
          run_engine_id: 'run-engine-b',
          run_id: 'run-a',
          status: 'completed',
          usable_output_count: 1,
        },
        {
          engine_id: 'unlimited-ocr-ffi',
          engine_kind: 'ocr',
          label: 'Unlimited OCR',
          ordinal: 0,
          parameters: { language: 'eng' },
          previewer: 'ocr_annotation',
          profile_id: 'profile-from-run',
          run_engine_id: 'run-engine-a',
          run_id: 'run-a',
          status: 'completed',
          usable_output_count: 1,
        },
      ],
      fixtureEnginePresets,
      'experimental-exact-prefill-q4',
    );

    expect(fromRun.map((item) => item.engineId)).toEqual(['unlimited-ocr-ffi', 'dots-mocr-gguf']);
    expect(fromRun[0]?.profileId).toBe('profile-from-run');
    expect(fromRun[1]?.parametersJson).toContain('"mode": "markdown"');
  });
});

const fixtureEnginePresets = [
  {
    availability: 'ready',
    available: true,
    default_enabled: true,
    default_parameters: { language: 'eng' },
    description: 'Fixture OCR engine',
    download_model_ids: ['unlimited-ocr-q4-k-m'],
    engine_id: 'unlimited-ocr-ffi',
    engine_kind: 'ocr',
    label: 'Unlimited OCR',
    model_id: 'unlimited-ocr-q4-k-m',
    parameter_schema: {},
    preset_id: 'ocr-unlimited-ocr-ffi',
    previewer: 'ocr_annotation',
    profile_id: 'experimental-exact-prefill-q4',
    requires_model: true,
    runner_detail: null,
    runner_kind: 'llama.cpp-ffi',
    runner_status: 'ready',
  },
  {
    availability: 'native_runner_missing',
    availability_detail: 'Build or install the native runner binary: llama-mtmd-cli.',
    available: false,
    default_enabled: false,
    default_parameters: {},
    description: 'Fixture document engine',
    download_model_ids: ['dots-mocr-gguf'],
    engine_id: 'dots-mocr-gguf',
    engine_kind: 'document_understanding',
    label: 'dots.mOCR',
    model_id: 'dots-mocr-gguf',
    parameter_schema: {},
    preset_id: 'du-dots-mocr-gguf',
    previewer: 'document_markdown',
    requires_model: true,
    runner_detail: 'uses llama-mtmd-cli with dots.ocr GGUF and mmproj assets',
    runner_kind: 'gguf-vlm-native',
    runner_status: 'wired',
  },
];
