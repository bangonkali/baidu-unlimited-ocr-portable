import * as Checkbox from '@radix-ui/react-checkbox';
import { Check, Download } from 'lucide-react';

import type { ModelsPayload, SettingsPayload } from '../../api/types';
import { IconButton } from '../../components/IconButton';
import { setLabelsVisible, setOverlayVisible } from '../../stores/workbenchStore';
import styles from './DetailsPane.module.css';

interface DetailsPaneProps {
  models?: ModelsPayload;
  settings?: SettingsPayload;
  selectedFileHash?: string;
  selectedRegionId?: string;
  overlayVisible: boolean;
  labelsVisible: boolean;
  modelDownloadBusy?: boolean;
  onDownloadModel?: (modelId: string) => void;
}

export function DetailsPane(props: DetailsPaneProps) {
  return (
    <aside className={styles.details} aria-label="Details">
      <div className={styles.header}>Details</div>
      <dl className={styles.meta}>
        <dt>Document</dt>
        <dd>{props.selectedFileHash ?? 'None'}</dd>
        <dt>Region</dt>
        <dd>{props.selectedRegionId ?? 'None'}</dd>
        <dt>PDF DPI</dt>
        <dd>{props.settings?.pdf_dpi ?? 200}</dd>
      </dl>
      <VisibilityControls
        labelsVisible={props.labelsVisible}
        overlayVisible={props.overlayVisible}
      />
      <ModelList
        busy={props.modelDownloadBusy}
        models={props.models}
        onDownloadModel={props.onDownloadModel}
      />
      <ProfileList models={props.models} />
    </aside>
  );
}

function VisibilityControls(props: Pick<DetailsPaneProps, 'labelsVisible' | 'overlayVisible'>) {
  return (
    <div className={styles.group}>
      <label className={styles.checkboxRow} htmlFor="overlay-visible">
        <Checkbox.Root
          checked={props.overlayVisible}
          className={styles.checkbox}
          id="overlay-visible"
          onCheckedChange={(checked) => setOverlayVisible(checked === true)}
        >
          <Checkbox.Indicator>
            <Check size={13} />
          </Checkbox.Indicator>
        </Checkbox.Root>
        Boxes
      </label>
      <label className={styles.checkboxRow} htmlFor="labels-visible">
        <Checkbox.Root
          checked={props.labelsVisible}
          className={styles.checkbox}
          id="labels-visible"
          onCheckedChange={(checked) => setLabelsVisible(checked === true)}
        >
          <Checkbox.Indicator>
            <Check size={13} />
          </Checkbox.Indicator>
        </Checkbox.Root>
        Labels
      </label>
    </div>
  );
}

function ModelList({
  busy,
  models,
  onDownloadModel,
}: Pick<DetailsPaneProps, 'models' | 'onDownloadModel'> & { busy?: boolean }) {
  return (
    <div className={styles.group}>
      <div className={styles.groupTitle}>Models</div>
      {models?.models.map((model) => (
        <div className={styles.model} key={model.model_id}>
          <div className={styles.modelText}>
            <span>{model.display_name}</span>
            <small title={model.error ?? model.local_path ?? undefined}>{model.status}</small>
          </div>
          <IconButton
            disabled={busy || model.status === 'downloaded' || model.status === 'downloading'}
            icon={Download}
            label={`Download ${model.display_name}`}
            onClick={() => onDownloadModel?.(model.model_id)}
          />
        </div>
      ))}
    </div>
  );
}

function ProfileList({ models }: Pick<DetailsPaneProps, 'models'>) {
  return (
    <div className={styles.group}>
      <div className={styles.groupTitle}>Profiles</div>
      {models?.profiles.map((profile) => (
        <div className={styles.profile} key={profile.key}>
          <span>{profile.label}</span>
          <small>{profile.default_max_tokens} tokens</small>
        </div>
      ))}
    </div>
  );
}
