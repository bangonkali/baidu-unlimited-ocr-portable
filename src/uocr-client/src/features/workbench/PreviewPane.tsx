import type { OverlayBox } from '../../api/types';
import { setSelection } from '../../stores/workbenchStore';
import styles from './PreviewPane.module.css';

interface PreviewPaneProps {
  boxes: OverlayBox[];
  labelsVisible: boolean;
  overlayVisible: boolean;
  selectedRegionId?: string;
}

export function PreviewPane({
  boxes,
  labelsVisible,
  overlayVisible,
  selectedRegionId,
}: PreviewPaneProps) {
  return (
    <section className={styles.preview} aria-label="Preview">
      <div className={styles.header}>Preview</div>
      <div className={styles.canvas}>
        <div className={styles.page}>
          <div className={styles.pageGrid} />
          {overlayVisible
            ? boxes.map((box) => (
                <button
                  className={styles.box}
                  data-active={box.region_id === selectedRegionId}
                  key={box.region_id}
                  onClick={() => setSelection({ pageNo: box.page_no, regionId: box.region_id })}
                  style={{
                    height: `${box.height_percent}%`,
                    left: `${box.left_percent}%`,
                    top: `${box.top_percent}%`,
                    width: `${box.width_percent}%`,
                  }}
                  type="button"
                >
                  {labelsVisible ? <span>{box.label}</span> : null}
                </button>
              ))
            : null}
        </div>
      </div>
    </section>
  );
}
