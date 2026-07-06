import type { WizardStepRecord } from './IngestWizardStepper';

export function ingestWizardSteps(args: {
  canStart: boolean;
  embeddingAfterIngest: boolean;
  embeddingReady: boolean;
  modelReady: boolean;
  rootReady: boolean;
  textIndexAfterIngest: boolean;
}): WizardStepRecord[] {
  const sourceReady = args.rootReady && args.modelReady;
  const embeddingBlocked = args.embeddingAfterIngest && !args.embeddingReady;
  return [
    {
      description: sourceReady ? 'Folder and OCR model ready' : 'Choose folder and OCR model',
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
