import { Crosshair } from 'lucide-react';
import type { RefObject } from 'react';
import { useEffect, useRef } from 'react';

import { annotationDomId, annotationIdOf } from '../../api/annotationIdentity';
import type { OverlayBox } from '../../api/types';
import styles from './PreviewPane.module.css';
import { needsRevealScroll } from './scrollVisibility';

interface ScrollGeometry {
  rootScroll: number;
  rootSize: number;
  rootStart: number;
  targetSize: number;
  targetStart: number;
}

export function centeredScrollOffset(geometry: ScrollGeometry) {
  return (
    geometry.rootScroll +
    geometry.targetStart -
    geometry.rootStart +
    geometry.targetSize / 2 -
    geometry.rootSize / 2
  );
}

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
  onSelectRegion: (pageNo: number, regionId: string) => void;
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
  onSelectRegion,
}: PreviewPaneProps) {
  const canvasRef = useRef<HTMLDivElement>(null);
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
      <div className={styles.canvas} ref={canvasRef}>
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
                scrollRootRef={canvasRef}
                selectedPageNo={selectedPageNo}
                selectedRegionId={selectedRegionId}
                onSelectRegion={onSelectRegion}
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
  scrollRootRef: RefObject<HTMLDivElement | null>;
  selectedPageNo: number;
  selectedRegionId?: string;
  onSelectRegion: (pageNo: number, regionId: string) => void;
}) {
  const pageRef = useRef<HTMLElement>(null);
  const activeBoxRef = useRef<HTMLButtonElement>(null);
  const selectedRegionId = props.selectedRegionId;
  const scrollRootRef = props.scrollRootRef;
  const imageUrl =
    props.getImageUrl?.(props.fileHash, props.pageNo) ??
    `/api/documents/${encodeURIComponent(props.fileHash)}/preview-images/source/${props.pageNo}`;

  useEffect(() => {
    if (props.pageNo !== props.selectedPageNo) {
      return;
    }
    const target = selectedRegionId ? (activeBoxRef.current ?? pageRef.current) : pageRef.current;
    const root = scrollRootRef.current;
    if (!target || !root) {
      return;
    }
    const rootRect = root.getBoundingClientRect();
    const targetRect = target.getBoundingClientRect();
    if (!needsRevealScroll(rootRect, targetRect)) {
      return;
    }
    root.scrollTo({
      behavior: 'smooth',
      left: centeredScrollOffset({
        rootScroll: root.scrollLeft,
        rootSize: rootRect.width,
        rootStart: rootRect.left,
        targetSize: targetRect.width,
        targetStart: targetRect.left,
      }),
      top: centeredScrollOffset({
        rootScroll: root.scrollTop,
        rootSize: rootRect.height,
        rootStart: rootRect.top,
        targetSize: targetRect.height,
        targetStart: targetRect.top,
      }),
    });
  }, [props.pageNo, props.selectedPageNo, scrollRootRef, selectedRegionId]);

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
          ? props.boxes.map((box) => {
              const annotationId = annotationIdOf(box);
              return (
                <button
                  className={styles.box}
                  data-active={annotationId === props.selectedRegionId}
                  data-annotation-id={annotationId}
                  data-region-id={box.region_id}
                  id={annotationDomId('annotation-box', annotationId)}
                  key={annotationId}
                  onClick={() => props.onSelectRegion(box.page_no, annotationId)}
                  ref={annotationId === props.selectedRegionId ? activeBoxRef : undefined}
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
              );
            })
          : null}
      </div>
    </article>
  );
}
