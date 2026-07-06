import type { useNavigate } from '@tanstack/react-router';

import type { useOpenFolderDialog, useStartIngest } from '../../api/hooks';
import type { ModelAssetRecord } from '../../api/types';
import {
  clearFolderDialogError,
  setAutoFollowRegions,
  setFolderDialogError,
  setSelectedRoot,
  setSelection,
} from '../../stores/workbenchStore';

export function useWorkbenchIngestActions(args: {
  folderDialog: ReturnType<typeof useOpenFolderDialog>;
  model?: ModelAssetRecord;
  navigate: ReturnType<typeof useNavigate>;
  engineId?: string;
  rootPath: string;
  runtimeId?: string;
  selectedProfile: string;
  startIngest: ReturnType<typeof useStartIngest>;
}) {
  const pickFolder = () => {
    clearFolderDialogError();
    return args.folderDialog
      .mutateAsync()
      .then((result) => {
        if (result.cancelled) {
          if (result.error) {
            setFolderDialogError(manualPathFallbackMessage(result.error));
          }
          return;
        }
        if (result.selected_path.trim()) {
          setSelectedRoot(result.selected_path);
          return;
        }
        setFolderDialogError(manualPathFallbackMessage('Folder picker returned an empty path'));
      })
      .catch((error: unknown) => {
        setFolderDialogError(manualPathFallbackMessage(errorMessage(error)));
      });
  };
  const startScan = (options?: { reprocess?: boolean }) => {
    clearFolderDialogError();
    void args.startIngest
      .mutateAsync({
        model_id: args.model?.model_id,
        profile_id: args.selectedProfile,
        reprocess: options?.reprocess ?? false,
        root_path: args.rootPath,
        ...(args.engineId ? { engine_id: args.engineId } : {}),
        ...(args.runtimeId ? { runtime_id: args.runtimeId } : {}),
      })
      .then((response) => {
        const firstFileHash = response.documents[0]?.file_hash ?? response.run.file_hashes?.[0];
        const pageNo = response.documents[0]?.current_page ?? 1;
        setAutoFollowRegions(true);
        setSelection({
          fileHash: firstFileHash,
          pageNo,
          regionId: undefined,
          runId: response.run.run_id,
        });
        void args.navigate({
          search: firstFileHash
            ? { file: firstFileHash, follow: true, page: pageNo, run: response.run.run_id }
            : {},
          to: '/workbench',
        });
      })
      .catch(() => undefined);
  };
  return { pickFolder, startScan };
}

function errorMessage(error: unknown) {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  return 'Folder picker failed';
}

function manualPathFallbackMessage(message: string) {
  const trimmed = message.trim() || 'Folder picker failed';
  const suffix = trimmed.endsWith('.') ? '' : '.';
  return `${trimmed}${suffix} Paste a folder path manually.`;
}
