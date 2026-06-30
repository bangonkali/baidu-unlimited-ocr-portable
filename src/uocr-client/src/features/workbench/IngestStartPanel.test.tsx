import { describe, expect, test } from 'bun:test';
import { renderToString } from 'react-dom/server';

import { fixtureModels, fixtureRuns } from '../../stories/fixtures/workbenchFixtures';
import { IngestStartPanel, isIngestBusy } from './IngestStartPanel';

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
        onModelChange={() => undefined}
        onPickFolder={() => undefined}
        onProfileChange={() => undefined}
        onRootPathChange={() => undefined}
        onStart={() => undefined}
        onStop={() => undefined}
        profiles={fixtureModels.profiles}
        rootPath="C:\\data\\incoming"
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
    expect(html).toContain('Choose Folder');
    expect(html).toContain('Unlimited-OCR Q4_K_M');
    expect(html).toContain('Stop Active Run');
    expect(html).toContain('disabled=""');
  });
});
