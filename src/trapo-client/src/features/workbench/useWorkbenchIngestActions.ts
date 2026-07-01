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
  const pickFolder = () => {
    void args.folderDialog.mutateAsync().then((result) => {
      if (!result.cancelled) {
        setSelectedRoot(result.selected_path);
      }
    });
  };
  const startScan = (options?: { reprocess?: boolean }) =>
    args.startIngest.mutate({
      model_id: args.model?.model_id,
      profile_id: args.selectedProfile,
      reprocess: options?.reprocess ?? false,
      root_path: args.rootPath,
    });
  return { pickFolder, startScan };
}
