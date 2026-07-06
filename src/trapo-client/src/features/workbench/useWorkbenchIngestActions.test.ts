import { describe, expect, test } from 'bun:test';

import {
  clearFolderDialogError,
  getWorkbenchSnapshot,
  setAutoFollowRegions,
  setSelectedRoot,
  setSelection,
} from '../../stores/workbenchStore';
import { fixtureModels } from '../../stories/fixtures/workbenchFixtures';
import { useWorkbenchIngestActions } from './useWorkbenchIngestActions';

describe('useWorkbenchIngestActions', () => {
  test('seeds follow selection and navigates to the started file', async () => {
    const navigateCalls: unknown[] = [];
    const startCalls: unknown[] = [];
    setAutoFollowRegions(false);
    setSelection({ fileHash: undefined, pageNo: 1, regionId: undefined });
    const actions = useWorkbenchIngestActions({
      folderDialog: { mutateAsync: async () => ({ cancelled: true }) } as never,
      model: fixtureModels.models[0],
      navigate: ((options: unknown) => {
        navigateCalls.push(options);
      }) as never,
      rootPath: '/data/incoming',
      selectedProfile: 'experimental-exact-prefill-q4',
      startIngest: {
        mutateAsync: async (payload: unknown) => {
          startCalls.push(payload);
          return {
            documents: [
              {
                display_name: 'invoice.pdf',
                file_hash: 'file-a',
                page_count: 1,
                relative_path: 'invoice.pdf',
                status: 'queued',
              },
            ],
            replay_since_sequence: 8,
            run: {
              file_hashes: ['file-a'],
              root_path: '/data/incoming',
              run_id: 'run-a',
              status: 'queued',
            },
          };
        },
      } as never,
    });

    actions.startScan({ reprocess: true });
    await Promise.resolve();

    expect(startCalls).toEqual([
      {
        model_id: fixtureModels.models[0]?.model_id,
        profile_id: 'experimental-exact-prefill-q4',
        reprocess: true,
        root_path: '/data/incoming',
      },
    ]);
    expect(navigateCalls).toEqual([
      { search: { file: 'file-a', follow: true, page: 1, run: 'run-a' }, to: '/workbench' },
    ]);
    expect(getWorkbenchSnapshot().autoFollowRegions).toBe(true);
    expect(getWorkbenchSnapshot().selection.fileHash).toBe('file-a');
    expect(getWorkbenchSnapshot().selection.runId).toBe('run-a');
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

  test('includes restart-prefilled engine and runtime when starting', async () => {
    const startCalls: unknown[] = [];
    const actions = useWorkbenchIngestActions({
      engineId: 'pdfium-unlimited-ocr',
      folderDialog: { mutateAsync: async () => ({ cancelled: true }) } as never,
      model: fixtureModels.models[0],
      navigate: (() => undefined) as never,
      rootPath: '/data/incoming',
      runtimeId: 'cuda',
      selectedProfile: 'experimental-exact-prefill-q4',
      startIngest: {
        mutateAsync: async (payload: unknown) => {
          startCalls.push(payload);
          return {
            documents: [],
            replay_since_sequence: 0,
            run: { root_path: '/data/incoming', run_id: 'run-a', status: 'queued' },
          };
        },
      } as never,
    });

    actions.startScan();
    await Promise.resolve();

    expect(startCalls).toEqual([
      {
        engine_id: 'pdfium-unlimited-ocr',
        model_id: fixtureModels.models[0]?.model_id,
        profile_id: 'experimental-exact-prefill-q4',
        reprocess: false,
        root_path: '/data/incoming',
        runtime_id: 'cuda',
      },
    ]);
  });
});
