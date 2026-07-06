import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import {
  fixtureDownloadedEmbeddingModel,
  fixtureModels,
  fixtureRuns,
} from '../../stories/fixtures/workbenchFixtures';
import { IngestStartPanel, isIngestBusy } from './IngestStartPanel';
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
});
