import { Download, RotateCcw, XCircle } from 'lucide-react';

import type { IngestEnginePresetRecord, ModelAssetRecord } from '../../api/types';
import wizardStyles from './IngestWizard.module.css';
import styles from './IngestWizardEnginePlan.module.css';
import {
  formatModelBytes,
  isModelDownloading,
  isModelReady,
  modelRequiredBytes,
  modelStatusLabel,
} from './ingestWizardModels';

interface EnginePlanModelActionsProps {
  busy?: boolean;
  models: ModelAssetRecord[];
  preset: IngestEnginePresetRecord;
  onCancelModel: (modelId: string) => void;
  onDownloadModel: (modelId: string, force?: boolean) => void;
}

export function EnginePlanModelActions({
  busy,
  models,
  onCancelModel,
  onDownloadModel,
  preset,
}: EnginePlanModelActionsProps) {
  const requiredModels = preset.download_model_ids
    .map((modelId) => models.find((model) => model.model_id === modelId))
    .filter((model): model is ModelAssetRecord => Boolean(model));
  if (requiredModels.length === 0) {
    return null;
  }
  return (
    <div className={styles.modelActions}>
      {requiredModels.map((model) => {
        const ready = isModelReady(model);
        const active = isModelDownloading(model);
        return (
          <div className={styles.modelRow} key={model.model_id}>
            <span>
              {model.display_name} · {modelStatusLabel(model)} ·{' '}
              {formatModelBytes(modelRequiredBytes(model))}
            </span>
            {active ? (
              <button
                className={wizardStyles.button}
                disabled={busy}
                onClick={() => onCancelModel(model.model_id)}
                type="button"
              >
                <XCircle size={15} />
                Cancel
              </button>
            ) : null}
            {!ready && !active ? (
              <button
                className={wizardStyles.button}
                disabled={busy}
                onClick={() => onDownloadModel(model.model_id)}
                type="button"
              >
                {model.status === 'failed' || model.status === 'cancelled' ? (
                  <RotateCcw size={15} />
                ) : (
                  <Download size={15} />
                )}
                Download
              </button>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}
