import {
  useCancelModelDownload,
  useDiagnosticProgress,
  useDocumentPreviewImages,
  useDocumentRegions,
  useDocuments,
  useDocumentText,
  useDownloadModel,
  useGenerateEmbedding,
  useIngestEngines,
  useIngestPreviewResults,
  useIngestRuns,
  useLogs,
  useModels,
  useOpenFolderDialog,
  useResumeRun,
  useRunCommand,
  useSelectModel,
  useSettings,
  useStartIngest,
  useStartTextIndex,
  useStatus,
  useUpdateSettings,
  useUsedEmbeddingModels,
} from '../../api/hooks';
import { engineResultOptions, selectedRunEngineIdFromOptions } from './engineResultOptions';
import { latestRunIdFromRuns } from './workbenchExplorerFilter';

interface WorkbenchDataArgs {
  debouncedSearch: string;
  fileHash?: string;
  resultId?: string;
  runId?: string;
  selectionRunEngineId?: string;
  selectionRunId?: string;
}

export function useWorkbenchData({
  debouncedSearch,
  fileHash,
  resultId,
  runId,
  selectionRunEngineId,
  selectionRunId,
}: WorkbenchDataArgs) {
  const runs = useIngestRuns();
  const documentRunId = runId ?? latestRunIdFromRuns(runs.data?.runs);
  const documents = useDocuments(debouncedSearch);
  const previewResults = useIngestPreviewResults(documentRunId, fileHash);
  const documentRun = runs.data?.runs.find((run) => run.run_id === documentRunId);
  const selectedDocument = documents.data?.documents.find(
    (document) => document.file_hash === fileHash,
  );
  const runPreviewResults = documentRun?.preview_results ?? [];
  const previewResultOptions = engineResultOptions({
    document: selectedDocument,
    results: previewResults.data?.results.length ? previewResults.data.results : runPreviewResults,
    run: documentRun,
  });
  const selectedRunEngineId = selectedRunEngineIdFromOptions({
    explicitResultId: resultId,
    results: previewResultOptions,
    selectionRunEngineId:
      !selectionRunId || selectionRunId === documentRunId ? selectionRunEngineId : undefined,
  });

  return {
    cancelModelDownload: useCancelModelDownload(),
    documents,
    documentRunId,
    downloadModel: useDownloadModel(),
    folderDialog: useOpenFolderDialog(),
    generateEmbedding: useGenerateEmbedding(),
    ingestEngines: useIngestEngines(),
    logs: useLogs(220),
    models: useModels(),
    previewImages: useDocumentPreviewImages(fileHash),
    previewResults,
    previewResultOptions,
    progress: useDiagnosticProgress(undefined, 5000, 1500),
    regions: useDocumentRegions(fileHash, documentRunId, selectedRunEngineId),
    resumeRun: useResumeRun(),
    runPreviewResults,
    runs,
    selectModel: useSelectModel(),
    selectedRunEngineId,
    selectedDocument,
    settings: useSettings(),
    startIngest: useStartIngest(),
    startTextIndex: useStartTextIndex(),
    status: useStatus(),
    stopRun: useRunCommand('stop'),
    text: useDocumentText(fileHash, documentRunId, selectedRunEngineId),
    updateSettings: useUpdateSettings(),
    usedEmbeddingModels: useUsedEmbeddingModels(),
  };
}
