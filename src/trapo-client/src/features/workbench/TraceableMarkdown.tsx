import type { ReactNode } from 'react';
import { useMemo } from 'react';
import type { Components } from 'react-markdown';

import { annotationTextDomId } from '../../api/annotationIdentity';
import type { OverlayBox, PageTextRecord } from '../../api/types';
import { MarkdownWithHtmlTables } from './HtmlTableMarkdown';
import styles from './TextPane.module.css';
import type { IndexedRegionAnchor, RegionAnchor } from './traceRegionAnchors';
import {
  indexedRegionAnchors,
  overlayRegionMap,
  REGION_MARKER,
  regionAnchors,
  regionIdFromHref,
  snippetFromRegion,
} from './traceRegionAnchors';

interface TraceableMarkdownProps {
  glowRegionId?: string;
  page: PageTextRecord;
  regions: OverlayBox[];
  selectedRegionId?: string;
  onRegionSelect: (pageNo: number, regionId: string) => void;
}

interface RegionScopeContext {
  glowRegionId?: string;
  onRegionSelect: (pageNo: number, regionId: string) => void;
  regionById: Map<string, OverlayBox>;
  selectedRegionId?: string;
}

interface MarkdownScopeContext extends RegionScopeContext {
  components: Components;
}

export function TraceableMarkdown({
  glowRegionId,
  onRegionSelect,
  page,
  regions,
  selectedRegionId,
}: TraceableMarkdownProps) {
  const anchors = useMemo(() => regionAnchors(page), [page]);
  const regionById = useMemo(() => overlayRegionMap(regions), [regions]);
  const scopes = useMemo(() => regionTextScopes(page.text, anchors), [anchors, page.text]);
  const components = useMemo(
    () => createTraceComponents(anchors, regionById, selectedRegionId, onRegionSelect),
    [anchors, onRegionSelect, regionById, selectedRegionId],
  );
  const context = { components, glowRegionId, onRegionSelect, regionById, selectedRegionId };
  return <div className={styles.markdownBody}>{renderMarkdownScopes(scopes, context)}</div>;
}

export function PlainTraceText({
  glowRegionId,
  onRegionSelect,
  page,
  regions,
  selectedRegionId,
}: TraceableMarkdownProps) {
  const anchors = useMemo(() => regionAnchors(page), [page]);
  const regionById = useMemo(() => overlayRegionMap(regions), [regions]);
  const context = { glowRegionId, onRegionSelect, regionById, selectedRegionId };
  return <p>{renderPlainTraceText(page.text, anchors, context)}</p>;
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
          onRegionSelect,
          pageNo: anchor?.pageNo ?? regionById.get(regionId)?.page_no ?? 0,
          region: regionById.get(regionId),
          regionId,
          selected: selectedRegionId === regionId,
        });
      }
      return (
        <a {...props} href={href} rel="noreferrer" target="_blank">
          {children}
        </a>
      );
    },
    p: ({ children }) => <>{children}</>,
    table: ({ children, node: _node, ...props }) => (
      <div className={styles.tableViewport}>
        <table {...props}>{children}</table>
      </div>
    ),
  } as Components;
}

function renderMarkdownScopes(scopes: RegionTextScopes, context: MarkdownScopeContext) {
  return [
    ...markdownLeadingNode(scopes.leadingText, context.components),
    ...scopes.scopes.map((scope) =>
      regionScope(scopeProps(scope, context, markdownScopeContent(scope, context.components))),
    ),
  ];
}

function renderPlainTraceText(text: string, anchors: RegionAnchor[], context: RegionScopeContext) {
  const scopes = regionTextScopes(text, anchors);
  if (scopes.scopes.length === 0) {
    return text;
  }
  return [
    ...plainLeadingNode(scopes.leadingText),
    ...scopes.scopes.map((scope) => regionScope(scopeProps(scope, context, scope.text))),
  ];
}

interface RegionTextScopes {
  leadingText: string;
  scopes: Array<{
    anchor: IndexedRegionAnchor;
    key: string;
    text: string;
  }>;
}

function regionTextScopes(text: string, anchors: RegionAnchor[]): RegionTextScopes {
  const indexedAnchors = indexedRegionAnchors(text, anchors);
  if (indexedAnchors.length === 0) {
    return { leadingText: text, scopes: [] };
  }
  return {
    leadingText: text.slice(0, indexedAnchors[0]?.index ?? 0),
    scopes: indexedAnchors.map((anchor, index) => {
      const next = indexedAnchors[index + 1];
      return {
        anchor,
        key: `${anchor.regionId}-${anchor.index}`,
        text: text.slice(anchor.index, next?.index ?? text.length),
      };
    }),
  };
}

function markdownLeadingNode(text: string, components: Components) {
  return text
    ? [<MarkdownWithHtmlTables components={components} key="leading" markdown={text} />]
    : [];
}

function markdownScopeContent(scope: RegionTextScopes['scopes'][number], components: Components) {
  return <MarkdownWithHtmlTables components={components} markdown={scope.text} />;
}

function plainLeadingNode(text: string) {
  return text ? [text] : [];
}

function scopeProps(
  scope: RegionTextScopes['scopes'][number],
  context: RegionScopeContext,
  content: ReactNode,
) {
  return {
    content,
    glowRegionId: context.glowRegionId,
    key: scope.key,
    onRegionSelect: context.onRegionSelect,
    pageNo: scope.anchor.pageNo,
    region: context.regionById.get(scope.anchor.regionId),
    regionId: scope.anchor.regionId,
    selectedRegionId: context.selectedRegionId,
  };
}

function regionScope({
  content,
  glowRegionId,
  key,
  onRegionSelect,
  pageNo,
  region,
  regionId,
  selectedRegionId,
}: {
  content: ReactNode;
  glowRegionId?: string;
  key: string;
  onRegionSelect: (pageNo: number, regionId: string) => void;
  pageNo: number;
  region?: OverlayBox;
  regionId: string;
  selectedRegionId?: string;
}) {
  const snippet = snippetFromRegion(region);
  const selected = selectedRegionId === regionId;
  return (
    <span className={styles.regionScope} data-glow={glowRegionId === regionId} key={key}>
      {regionAnchorButton({ onRegionSelect, pageNo, region, regionId, selected })}
      <span className={styles.regionScopeText}>{content}</span>
      {snippet ? (
        <button
          aria-label={`Focus ${regionLabel(region, 'annotation region')} image`}
          className={styles.regionSnippetButton}
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

function regionAnchorButton({
  onRegionSelect,
  pageNo,
  region,
  regionId,
  selected,
}: {
  onRegionSelect: (pageNo: number, regionId: string) => void;
  pageNo: number;
  region?: OverlayBox;
  regionId: string;
  selected: boolean;
}) {
  const label = regionLabel(region, 'annotation region');
  return (
    <button
      aria-label={`Focus ${label}`}
      className={styles.regionAnchor}
      data-active={selected}
      id={annotationTextDomId(regionId)}
      onClick={() => onRegionSelect(pageNo, regionId)}
      title={label}
      type="button"
    >
      {REGION_MARKER}
    </button>
  );
}

function regionLabel(region: OverlayBox | undefined, fallback: string) {
  return region?.label?.trim() || fallback;
}
