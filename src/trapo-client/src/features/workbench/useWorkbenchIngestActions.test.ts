import { describe, expect, test } from 'bun:test';

import { fixtureModels } from '../../stories/fixtures/workbenchFixtures';
import { useWorkbenchIngestActions } from './useWorkbenchIngestActions';

describe('useWorkbenchIngestActions', () => {
  test('navigates to workbench after an ingest starts', () => {
    const navigateCalls: unknown[] = [];
    const startCalls: unknown[] = [];
    const actions = useWorkbenchIngestActions({
      folderDialog: { mutateAsync: async () => ({ cancelled: true }) } as never,
      model: fixtureModels.models[0],
      navigate: ((options: unknown) => {
        navigateCalls.push(options);
      }) as never,
      rootPath: '/data/incoming',
      selectedProfile: 'experimental-exact-prefill-q4',
      startIngest: {
        mutate: (payload: unknown, options: { onSuccess: () => void }) => {
          startCalls.push(payload);
          options.onSuccess();
        },
      } as never,
    });

    actions.startScan({ reprocess: true });

    expect(startCalls).toEqual([
      {
        model_id: fixtureModels.models[0]?.model_id,
        profile_id: 'experimental-exact-prefill-q4',
        reprocess: true,
        root_path: '/data/incoming',
      },
    ]);
    expect(navigateCalls).toEqual([{ to: '/workbench' }]);
  });
});
