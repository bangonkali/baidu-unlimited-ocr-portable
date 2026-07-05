import type { ReactNode } from 'react';
import { useMemo } from 'react';
import type { Components } from 'react-markdown';

import { annotationDomId } from '../../api/annotationIdentity';
import type { OverlayBox, PageTextRecord } from '../../api/types';
import { MarkdownWithHtmlTables } from './HtmlTableMarkdown';
import styles from './TextPane.module.css';
import type { RegionAnchor } from './traceRegionAnchors';
import {
  indexedRegionAnchors,
  injectRegionAnchors,
  overlayRegionMap,
  REGION_MARKER,
  regionAnchors,
  regionIdFromHref,
  snippetFromRegion,
} from './traceRegionAnchors';

interface TraceableMarkdownProps {
  page: PageTextRecord;
  regions: OverlayBox[];
  selectedRegionId?: string;
  onRegionSelect: (pageNo: number, regionId: string) => void;
}

export function TraceableMarkdown({
  onRegionSelect,
  page,
  regions,
  selectedRegionId,
}: TraceableMarkdownProps) {
  const anchors = useMemo(() => regionAnchors(page), [page]);
  const regionById = useMemo(() => overlayRegionMap(regions), [regions]);
  const markdown = useMemo(() => injectRegionAnchors(page.text, anchors), [anchors, page.text]);
  const components = useMemo(
    () => createTraceComponents(anchors, regionById, selectedRegionId, onRegionSelect),
    [anchors, onRegionSelect, regionById, selectedRegionId],
  );
  return (
    <div className={styles.markdownBody}>
      <MarkdownWithHtmlTables components={components} markdown={markdown} />
    </div>
  );
}

export function PlainTraceText({
  onRegionSelect,
  page,
  regions,
  selectedRegionId,
}: TraceableMarkdownProps) {
  const anchors = useMemo(() => regionAnchors(page), [page]);
  const regionById = useMemo(() => overlayRegionMap(regions), [regions]);
  return (
    <p>{renderPlainTraceText(page.text, anchors, regionById, selectedRegionId, onRegionSelect)}</p>
  );
}

function createTraceComponents(
  anchors: RegionAnchor[],
  regionById: Map<string, OverlayBox>,
  selectedRegionId: string | undefined,
  onRegionSelect: (pageNo: number, regionId: string) => void,
): Components {
  const anchorById = new Map(anchors.map((anchor) => [anchor.regionId, anchor]));
  return {
    a: ({ children, href, node: _node, ...props }) => {
      const regionId = regionIdFromHref(href);
      if (regionId) {
        const anchor = anchorById.get(regionId);
        return regionAnchorButton({
          key: `${regionId}-${anchor?.start ?? 0}`,
          onRegionSelect,
          pageNo: anchor?.pageNo ?? regionById.get(regionId)?.page_no ?? 0,
          region: regionById.get(regionId),
          regionId,
          selectedRegionId,
        });
      }
      return (
        <a {...props} href={href} rel="noreferrer" target="_blank">
          {children}
        </a>
      );
    },
    table: ({ children, node: _node, ...props }) => (
      <div className={styles.tableViewport}>
        <table {...props}>{children}</table>
      </div>
    ),
  } as Components;
}

function renderPlainTraceText(
  text: string,
  anchors: RegionAnchor[],
  regionById: Map<string, OverlayBox>,
  selectedRegionId: string | undefined,
  onRegionSelect: (pageNo: number, regionId: string) => void,
) {
  if (anchors.length === 0) {
    return text;
  }
  const nodes: ReactNode[] = [];
  let cursor = 0;
  for (const anchor of indexedRegionAnchors(text, anchors)) {
    if (anchor.index > cursor) {
      nodes.push(text.slice(cursor, anchor.index));
    }
    nodes.push(
      regionAnchorButton({
        key: `${anchor.regionId}-${anchor.index}`,
        onRegionSelect,
        pageNo: anchor.pageNo,
        region: regionById.get(anchor.regionId),
        regionId: anchor.regionId,
        selectedRegionId,
      }),
    );
    cursor = anchor.index;
  }
  if (cursor < text.length) {
    nodes.push(text.slice(cursor));
  }
  return nodes;
}

function regionAnchorButton({
  key,
  onRegionSelect,
  pageNo,
  region,
  regionId,
  selectedRegionId,
}: {
  key: string;
  onRegionSelect: (pageNo: number, regionId: string) => void;
  pageNo: number;
  region?: OverlayBox;
  regionId: string;
  selectedRegionId?: string;
}) {
  const snippet = snippetFromRegion(region);
  const sourceRegionId = region?.region_id ?? regionId;
  return (
    <span className={styles.anchorBundle} key={key}>
      <button
        aria-label={`Region ${region?.label || regionId}`}
        className={styles.regionAnchor}
        data-active={selectedRegionId === regionId}
        data-annotation-id={regionId}
        data-region-id={sourceRegionId}
        id={annotationDomId('annotation-text', regionId)}
        onClick={() => onRegionSelect(pageNo, regionId)}
        title={region?.label || regionId}
        type="button"
      >
        {REGION_MARKER}
      </button>
      {snippet ? (
        <button
          aria-label={`Focus region image ${region?.label || regionId}`}
          className={styles.regionSnippetButton}
          data-annotation-id={regionId}
          data-region-id={sourceRegionId}
          onClick={() => onRegionSelect(pageNo, regionId)}
          type="button"
        >
          <img
            alt={snippet.alt || region?.label || 'Region snippet'}
            className={styles.regionSnippet}
            loading="lazy"
            src={snippet.src}
          />
        </button>
      ) : null}
    </span>
  );
}
