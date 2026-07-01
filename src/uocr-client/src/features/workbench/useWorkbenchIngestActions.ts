import { useState } from 'react';

import type { useOpenFolderDialog, useStartIngest } from '../../api/hooks';
import type { ModelAssetRecord } from '../../api/types';
import { setSelectedRoot } from '../../stores/workbenchStore';

export function useWorkbenchIngestActions(args: {
  folderDialog: ReturnType<typeof useOpenFolderDialog>;
  model?: ModelAssetRecord;
  rootPath: string;
  selectedProfile: string;
  startIngest: ReturnType<typeof useStartIngest>;
}) {
  const [folderDialogError, setFolderDialogError] = useState<string | undefined>();
  const pickFolder = () => {
    setFolderDialogError(undefined);
    void args.folderDialog
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
    setFolderDialogError(undefined);
    args.startIngest.mutate({
      model_id: args.model?.model_id,
      profile_id: args.selectedProfile,
      reprocess: options?.reprocess ?? false,
      root_path: args.rootPath,
    });
  };
  return {
    clearFolderDialogError: () => setFolderDialogError(undefined),
    folderDialogError,
    pickFolder,
    startScan,
  };
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
