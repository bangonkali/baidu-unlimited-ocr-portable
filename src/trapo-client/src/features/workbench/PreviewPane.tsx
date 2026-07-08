import type { RefObject } from 'react';
import { useCallback, useEffect, useRef } from 'react';

import { annotationBoxDomId, annotationIdOf } from '../../api/annotationIdentity';
import type { IngestPreviewResultRecord, OverlayBox } from '../../api/types';
import { revealWhenAvailable } from './deferredReveal';
import { EmptyPreviewState } from './emptyPreviewState';
import { useFocusEmphasis } from './focusEmphasis';
import { overlayShapeForBox } from './overlayGeometry';
import styles from './PreviewPane.module.css';
import { PreviewSettingsMenu } from './PreviewSettingsMenu';
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

export function nearestScrollOffset(geometry: ScrollGeometry) {
  const rootEnd = geometry.rootStart + geometry.rootSize;
  const targetEnd = geometry.targetStart + geometry.targetSize;
  if (geometry.targetStart < geometry.rootStart || geometry.targetSize > geometry.rootSize) {
    return geometry.rootScroll + geometry.targetStart - geometry.rootStart;
  }
  if (targetEnd > rootEnd) {
    return geometry.rootScroll + targetEnd - rootEnd;
  }
  return geometry.rootScroll;
}

interface PreviewPaneProps {
  autoFollowRegions: boolean;
  boxes: OverlayBox[];
  fileHash?: string;
  focusRevision?: number;
  getImageUrl?: (fileHash: string, pageNo: number) => string;
  engineResults: IngestPreviewResultRecord[];
  labelsVisible: boolean;
  overlayVisible: boolean;
  pages: number[];
  selectedPageNo: number;
  selectedRunEngineId?: string;
  selectedRegionId?: string;
  onAutoFollowChange: (enabled: boolean) => void;
  onSelectPreviewResult: (runEngineId: string) => void;
  onSelectRegion: (pageNo: number, regionId: string) => void;
}

export function PreviewPane({
  autoFollowRegions,
  boxes,
  engineResults,
  fileHash,
  focusRevision = 0,
  getImageUrl,
  labelsVisible,
  overlayVisible,
  pages,
  selectedPageNo,
  selectedRunEngineId,
  selectedRegionId,
  onAutoFollowChange,
  onSelectPreviewResult,
  onSelectRegion,
}: PreviewPaneProps) {
  const canvasRef = useRef<HTMLDivElement>(null);
  const selectedResult = engineResults.find(
    (result) => result.run_engine_id === selectedRunEngineId,
  );
  return (
    <section className={styles.preview} aria-label="Preview" data-tour="preview">
      <div className={styles.header}>
        <span>Preview</span>
        <PreviewSettingsMenu
          autoFollowRegions={autoFollowRegions}
          engineResults={engineResults}
          selectedRunEngineId={selectedRunEngineId}
          onAutoFollowChange={onAutoFollowChange}
          onSelectPreviewResult={onSelectPreviewResult}
        />
      </div>
      <div className={styles.canvas} ref={canvasRef}>
        {!fileHash || pages.length === 0 ? (
          <EmptyPreviewState
            title={selectedResult ? 'No preview output yet' : 'No engine output selected'}
            detail={
              selectedResult
                ? `${selectedResult.label} has not produced a previewable page for this document yet.`
                : 'Run an engine or select an engine output to preview OCR overlays.'
            }
          />
        ) : null}
        {fileHash
          ? pages.map((pageNo) => (
              <PagePreview
                boxes={boxes.filter((box) => box.page_no === pageNo)}
                fileHash={fileHash}
                focusRevision={focusRevision}
                getImageUrl={getImageUrl}
                key={pageNo}
                labelsVisible={labelsVisible}
                overlayVisible={overlayVisible}
                pageNo={pageNo}
                scrollRootRef={canvasRef}
                selectedPageNo={selectedPageNo}
                selectedPageKey={`${fileHash}:${selectedRunEngineId ?? 'none'}:${selectedPageNo}`}
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
  focusRevision: number;
  getImageUrl?: (fileHash: string, pageNo: number) => string;
  labelsVisible: boolean;
  overlayVisible: boolean;
  pageNo: number;
  scrollRootRef: RefObject<HTMLDivElement | null>;
  selectedPageNo: number;
  selectedPageKey: string;
  selectedRegionId?: string;
  onSelectRegion: (pageNo: number, regionId: string) => void;
}) {
  const pageRef = useRef<HTMLElement>(null);
  const selectedRegionId = props.selectedRegionId;
  const scrollRootRef = props.scrollRootRef;
  const focusPageKey = useFocusEmphasis(props.selectedPageKey, 1);
  const revealSelectedPage = useCallback(() => {
    if (props.pageNo !== props.selectedPageNo) {
      return;
    }
    const root = scrollRootRef.current;
    const target = pageRef.current;
    if (root && target) {
      revealPreviewTarget(root, target, false);
    }
  }, [props.pageNo, props.selectedPageNo, scrollRootRef]);
  const imageUrl =
    props.getImageUrl?.(props.fileHash, props.pageNo) ??
    `/api/documents/${encodeURIComponent(props.fileHash)}/preview-images/source/${props.pageNo}`;

  useEffect(() => {
    if (props.pageNo !== props.selectedPageNo) {
      return undefined;
    }
    const revealGeneration = props.focusRevision;
    const root = scrollRootRef.current;
    if (!root) {
      return undefined;
    }
    if (selectedRegionId && revealGeneration >= 0) {
      const pageTarget = pageRef.current;
      if (pageTarget) {
        revealPreviewTarget(root, pageTarget, false);
      }
      return revealWhenAvailable({
        findTarget: () => findAnnotationBox(root, selectedRegionId),
        reveal: (target) => revealPreviewTarget(root, target, true),
        root,
      });
    }
    const target = pageRef.current;
    if (!target) {
      return undefined;
    }
    revealSelectedPage();
    return undefined;
  }, [
    props.focusRevision,
    props.pageNo,
    props.selectedPageNo,
    revealSelectedPage,
    scrollRootRef,
    selectedRegionId,
  ]);

  return (
    <article
      className={styles.pageBlock}
      data-active={props.pageNo === props.selectedPageNo}
      data-focus-emphasis={
        focusPageKey === props.selectedPageKey && props.pageNo === props.selectedPageNo
      }
      ref={pageRef}
    >
      <div className={styles.pageLabel}>Page {props.pageNo}</div>
      <div className={styles.page}>
        <img
          alt={`Page ${props.pageNo}`}
          className={styles.image}
          onLoad={revealSelectedPage}
          src={imageUrl}
        />
        {props.overlayVisible
          ? props.boxes.map((box) => {
              const annotationId = annotationIdOf(box);
              const shape = overlayShapeForBox(box);
              return (
                <button
                  className={styles.box}
                  data-active={annotationId === props.selectedRegionId}
                  data-shape={shape.isPolygon ? 'polygon' : 'axis-aligned'}
                  id={annotationBoxDomId(annotationId)}
                  key={annotationId}
                  onClick={() => props.onSelectRegion(box.page_no, annotationId)}
                  style={{
                    height: `${shape.bounds.height}%`,
                    left: `${shape.bounds.left}%`,
                    top: `${shape.bounds.top}%`,
                    width: `${shape.bounds.width}%`,
                  }}
                  type="button"
                >
                  {shape.isPolygon ? (
                    <svg
                      aria-hidden="true"
                      className={styles.boxShape}
                      focusable="false"
                      preserveAspectRatio="none"
                      viewBox="0 0 100 100"
                    >
                      <polygon points={shape.svgPoints} />
                    </svg>
                  ) : null}
                  {props.labelsVisible ? <span>{box.label}</span> : null}
                </button>
              );
            })
          : null}
      </div>
    </article>
  );
}

function revealPreviewTarget(root: HTMLElement, target: HTMLElement, centered: boolean) {
  const rootRect = root.getBoundingClientRect();
  const targetRect = target.getBoundingClientRect();
  if (!needsRevealScroll(rootRect, targetRect)) {
    return;
  }
  const scrollOffset = centered ? centeredScrollOffset : nearestScrollOffset;
  root.scrollTo({
    behavior: 'smooth',
    left: scrollOffset({
      rootScroll: root.scrollLeft,
      rootSize: rootRect.width,
      rootStart: rootRect.left,
      targetSize: targetRect.width,
      targetStart: targetRect.left,
    }),
    top: scrollOffset({
      rootScroll: root.scrollTop,
      rootSize: rootRect.height,
      rootStart: rootRect.top,
      targetSize: targetRect.height,
      targetStart: targetRect.top,
    }),
  });
}

export function findAnnotationBox(root: HTMLElement, annotationId: string) {
  const element = root.ownerDocument.getElementById(annotationBoxDomId(annotationId));
  return element instanceof HTMLElement && root.contains(element) ? element : null;
}
