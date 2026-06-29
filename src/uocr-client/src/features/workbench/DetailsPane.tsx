import * as Checkbox from '@radix-ui/react-checkbox';
import { Check } from 'lucide-react';

import type { DocumentSummary } from '../../api/types';
import { setLabelsVisible, setOverlayVisible } from '../../stores/workbenchStore';
import styles from './DetailsPane.module.css';

interface DetailsPaneProps {
  document?: DocumentSummary;
  labelsVisible: boolean;
  overlayVisible: boolean;
  selectedRegionId?: string;
}

export function DetailsPane(props: DetailsPaneProps) {
  return (
    <aside className={styles.details} aria-label="Details">
      <div className={styles.header}>Details</div>
      <dl className={styles.meta}>
        <dt>Document</dt>
        <dd>{props.document?.display_name ?? 'None'}</dd>
        <dt>Status</dt>
        <dd>{props.document?.status ?? 'No selection'}</dd>
        <dt>Pages</dt>
        <dd>{props.document?.page_count ?? 0}</dd>
        <dt>Regions</dt>
        <dd>{props.document?.regions ?? 0}</dd>
        <dt>Region</dt>
        <dd>{props.selectedRegionId ?? 'None'}</dd>
      </dl>
      <VisibilityControls
        labelsVisible={props.labelsVisible}
        overlayVisible={props.overlayVisible}
      />
      {props.document?.error ? <div className={styles.error}>{props.document.error}</div> : null}
    </aside>
  );
}

function VisibilityControls(props: Pick<DetailsPaneProps, 'labelsVisible' | 'overlayVisible'>) {
  return (
    <div className={styles.group}>
      <div className={styles.groupTitle}>Overlays</div>
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
