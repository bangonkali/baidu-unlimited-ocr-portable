import * as Checkbox from '@radix-ui/react-checkbox';
import { Check } from 'lucide-react';

import type { DocumentSummary, OverlayBox } from '../../api/types';
import { setLabelsVisible, setOverlayVisible } from '../../stores/workbenchStore';
import styles from './DetailsPane.module.css';
import { documentPageLabel, percentLabel } from './progressFormat';

interface DetailsPaneProps {
  document?: DocumentSummary;
  labelsVisible: boolean;
  overlayVisible: boolean;
  selectedRegion?: OverlayBox;
  selectedRegionContent?: string;
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
        <dt>Progress</dt>
        <dd>
          {props.document
            ? `${documentPageLabel(props.document)} · ${percentLabel(props.document.progress_percent)}`
            : 'No selection'}
        </dd>
        <dt>Regions</dt>
        <dd>{props.document?.regions ?? 0}</dd>
        <dt>Region</dt>
        <dd>{props.selectedRegionId ?? 'None'}</dd>
      </dl>
      <RegionDetails content={props.selectedRegionContent} region={props.selectedRegion} />
      <VisibilityControls
        labelsVisible={props.labelsVisible}
        overlayVisible={props.overlayVisible}
      />
      {props.document?.error ? <div className={styles.error}>{props.document.error}</div> : null}
    </aside>
  );
}

function RegionDetails({ content, region }: { content?: string; region?: OverlayBox }) {
  if (!region) {
    return (
      <div className={styles.regionDetails}>
        <div className={styles.groupTitle}>Selected Box</div>
        <div className={styles.empty}>None</div>
      </div>
    );
  }
  return (
    <div className={styles.regionDetails}>
      <div className={styles.groupTitle}>Selected Box</div>
      <dl className={styles.regionMeta}>
        <dt>Page</dt>
        <dd>{region.page_no}</dd>
        <dt>Label</dt>
        <dd>{region.label}</dd>
      </dl>
      <pre className={styles.regionContent}>{content || region.label || region.region_id}</pre>
    </div>
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
