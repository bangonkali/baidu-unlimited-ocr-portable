import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import {
  fixtureDownloadedEmbeddingModel,
  fixtureModels,
  fixtureRuns,
} from '../../stories/fixtures/workbenchFixtures';
import { IngestStartPanel, isIngestBusy } from './IngestStartPanel';
import { defaultEnginePlan } from './ingestEnginePlan';
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
        {
          engine_id: 'dots-mocr-gguf',
          engine_kind: 'document_understanding',
          ordinal: 1,
          preset_id: 'du-dots-mocr-gguf',
        },
      ],
      reprocess: true,
    });
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
  },
  {
    availability: 'fallback_adapter',
    available: true,
    default_enabled: true,
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
  },
];
