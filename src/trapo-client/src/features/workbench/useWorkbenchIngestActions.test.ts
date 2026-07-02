import { describe, expect, test } from 'bun:test';

import {
  clearFolderDialogError,
  getWorkbenchSnapshot,
  setSelectedRoot,
} from '../../stores/workbenchStore';
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

  test('stores manual path fallback when folder picker reports an error', async () => {
    clearFolderDialogError();
    const actions = useWorkbenchIngestActions({
      folderDialog: {
        mutateAsync: async () => ({
          cancelled: true,
          error: 'native Linux folder dialog requires zenity or kdialog',
        }),
      } as never,
      model: fixtureModels.models[0],
      navigate: (() => undefined) as never,
      rootPath: '',
      selectedProfile: 'experimental-exact-prefill-q4',
      startIngest: { mutate: () => undefined } as never,
    });

    await actions.pickFolder();

    expect(getWorkbenchSnapshot().folderDialogError).toBe(
      'native Linux folder dialog requires zenity or kdialog. Paste a folder path manually.',
    );
  });

  test('stores selected folder when picker succeeds', async () => {
    clearFolderDialogError();
    setSelectedRoot('');
    const actions = useWorkbenchIngestActions({
      folderDialog: {
        mutateAsync: async () => ({
          cancelled: false,
          selected_path: '/data/incoming',
        }),
      } as never,
      model: fixtureModels.models[0],
      navigate: (() => undefined) as never,
      rootPath: '',
      selectedProfile: 'experimental-exact-prefill-q4',
      startIngest: { mutate: () => undefined } as never,
    });

    await actions.pickFolder();

    expect(getWorkbenchSnapshot().selectedRoot).toBe('/data/incoming');
    expect(getWorkbenchSnapshot().folderDialogError).toBeUndefined();
  });
});
