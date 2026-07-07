import type { useNavigate } from '@tanstack/react-router';

import type { useOpenFolderDialog, useStartIngest } from '../../api/hooks';
import type { IngestEngineSelection, ModelAssetRecord } from '../../api/types';
import {
  clearFolderDialogError,
  setAutoFollowRegions,
  setFolderDialogError,
  setSelectedRoot,
  setSelection,
} from '../../stores/workbenchStore';

export interface StartScanOptions {
  embeddingAfterIngest?: boolean;
  embeddingDimension?: number;
  embeddingModelId?: string;
  engines?: IngestEngineSelection[];
  reprocess?: boolean;
  textIndexAfterIngest?: boolean;
}

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
  const startScan = (options?: StartScanOptions) => {
    clearFolderDialogError();
    void args.startIngest
      .mutateAsync({
        model_id: args.model?.model_id,
        profile_id: args.selectedProfile,
        reprocess: options?.reprocess ?? false,
        root_path: args.rootPath,
        ...(options?.textIndexAfterIngest ? { text_index_after_ingest: true } : {}),
        ...(options?.embeddingAfterIngest ? { embedding_after_ingest: true } : {}),
        ...(options?.embeddingModelId ? { embedding_model_id: options.embeddingModelId } : {}),
        ...(options?.embeddingDimension ? { embedding_dimension: options.embeddingDimension } : {}),
        ...(options?.engines && options.engines.length > 0 ? { engines: options.engines } : {}),
        ...(args.engineId ? { engine_id: args.engineId } : {}),
        ...(args.runtimeId ? { runtime_id: args.runtimeId } : {}),
      })
      .then((response) => {
        const firstFileHash = response.documents[0]?.file_hash ?? response.run.file_hashes?.[0];
        const pageNo = response.documents[0]?.current_page ?? 1;
        const firstRunEngineId = response.run.engine_configs?.[0]?.run_engine_id;
        setAutoFollowRegions(true);
        setSelection({
          fileHash: firstFileHash,
          pageNo,
          regionId: undefined,
          runEngineId: firstRunEngineId,
          runId: response.run.run_id,
        });
        void args.navigate({
          search: firstFileHash
            ? {
                file: firstFileHash,
                follow: true,
                page: pageNo,
                result: firstRunEngineId,
                run: response.run.run_id,
              }
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
