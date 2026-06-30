import { Crosshair } from 'lucide-react';
import { useEffect, useRef } from 'react';

import type { OverlayBox } from '../../api/types';
import { setSelection } from '../../stores/workbenchStore';
import styles from './PreviewPane.module.css';

interface PreviewPaneProps {
  autoFollowRegions: boolean;
  boxes: OverlayBox[];
  fileHash?: string;
  getImageUrl?: (fileHash: string, pageNo: number) => string;
  labelsVisible: boolean;
  overlayVisible: boolean;
  pages: number[];
  selectedPageNo: number;
  selectedRegionId?: string;
  onAutoFollowChange: (enabled: boolean) => void;
}

export function PreviewPane({
  autoFollowRegions,
  boxes,
  fileHash,
  getImageUrl,
  labelsVisible,
  overlayVisible,
  pages,
  selectedPageNo,
  selectedRegionId,
  onAutoFollowChange,
}: PreviewPaneProps) {
  return (
    <section className={styles.preview} aria-label="Preview" data-tour="preview">
      <div className={styles.header}>
        <span>Preview</span>
        <button
          aria-pressed={autoFollowRegions}
          className={styles.followToggle}
          onClick={() => onAutoFollowChange(!autoFollowRegions)}
          type="button"
        >
          <Crosshair size={14} strokeWidth={1.9} />
          <span>{autoFollowRegions ? 'Auto Follow On' : 'Auto Follow Off'}</span>
        </button>
      </div>
      <div className={styles.canvas}>
        {!fileHash || pages.length === 0 ? (
          <div className={styles.empty}>No preview yet</div>
        ) : null}
        {fileHash
          ? pages.map((pageNo) => (
              <PagePreview
                boxes={boxes.filter((box) => box.page_no === pageNo)}
                fileHash={fileHash}
                getImageUrl={getImageUrl}
                key={pageNo}
                labelsVisible={labelsVisible}
                overlayVisible={overlayVisible}
                pageNo={pageNo}
                selectedPageNo={selectedPageNo}
                selectedRegionId={selectedRegionId}
              />
            ))
          : null}
      </div>
    </section>
  );
}

function PagePreview(props: {
  boxes: OverlayBox[];
  fileHash: string;
  getImageUrl?: (fileHash: string, pageNo: number) => string;
  labelsVisible: boolean;
  overlayVisible: boolean;
  pageNo: number;
  selectedPageNo: number;
  selectedRegionId?: string;
}) {
  const pageRef = useRef<HTMLElement>(null);
  const activeBoxRef = useRef<HTMLButtonElement>(null);
  const selectedRegionId = props.selectedRegionId;
  const imageUrl =
    props.getImageUrl?.(props.fileHash, props.pageNo) ??
    `/api/documents/${encodeURIComponent(props.fileHash)}/preview-images/source/${props.pageNo}`;

  useEffect(() => {
    if (props.pageNo !== props.selectedPageNo) {
      return;
    }
    const target = selectedRegionId ? (activeBoxRef.current ?? pageRef.current) : pageRef.current;
    target?.scrollIntoView({ block: 'center', inline: 'center', behavior: 'smooth' });
  }, [props.pageNo, props.selectedPageNo, selectedRegionId]);

  return (
    <article
      className={styles.pageBlock}
      data-active={props.pageNo === props.selectedPageNo}
      ref={pageRef}
    >
      <div className={styles.pageLabel}>Page {props.pageNo}</div>
      <div className={styles.page}>
        <img alt={`Page ${props.pageNo}`} className={styles.image} src={imageUrl} />
        {props.overlayVisible
          ? props.boxes.map((box) => (
              <button
                className={styles.box}
                data-active={box.region_id === props.selectedRegionId}
                key={box.region_id}
                onClick={() => setSelection({ pageNo: box.page_no, regionId: box.region_id })}
                ref={box.region_id === props.selectedRegionId ? activeBoxRef : undefined}
                style={{
                  height: `${box.height_percent}%`,
                  left: `${box.left_percent}%`,
                  top: `${box.top_percent}%`,
                  width: `${box.width_percent}%`,
                }}
                type="button"
              >
                {props.labelsVisible ? <span>{box.label}</span> : null}
              </button>
            ))
          : null}
      </div>
    </article>
  );
}
