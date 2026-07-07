import type { WizardStepRecord } from './IngestWizardStepper';

export function ingestWizardSteps(args: {
  canStart: boolean;
  embeddingAfterIngest: boolean;
  embeddingReady: boolean;
  modelReady: boolean;
  planIssue?: string;
  planReady?: boolean;
  rootReady: boolean;
  textIndexAfterIngest: boolean;
}): WizardStepRecord[] {
  const enginesReady = args.planReady ?? args.modelReady;
  const sourceReady = args.rootReady && enginesReady;
  const embeddingBlocked = args.embeddingAfterIngest && !args.embeddingReady;
  return [
    {
      description: sourceReady
        ? 'Folder and engines ready'
        : (args.planIssue ?? 'Choose folder and OCR model'),
      label: 'Start Ingest',
      status: sourceReady ? 'ready' : 'blocked',
    },
    {
      description: args.textIndexAfterIngest ? 'Runs after OCR' : 'Optional',
      label: 'Text Index',
      status: sourceReady ? 'ready' : 'pending',
    },
    {
      description: args.embeddingAfterIngest
        ? 'Semantic vectors enabled'
        : 'Recommended optional step',
      label: 'Generate Embedding',
      status: embeddingBlocked ? 'blocked' : args.canStart ? 'current' : 'pending',
    },
  ];
}
