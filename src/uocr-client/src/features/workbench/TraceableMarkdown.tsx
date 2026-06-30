import type { ReactElement, ReactNode } from 'react';
import { Children, cloneElement, createElement, isValidElement, useMemo } from 'react';
import type { Components } from 'react-markdown';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

import type { PageTextRecord, TextRegionSpan } from '../../api/types';
import styles from './TextPane.module.css';

interface TraceableMarkdownProps {
  page: PageTextRecord;
  selectedRegionId?: string;
  onRegionSelect: (pageNo: number, regionId: string) => void;
}

interface TraceCandidate {
  end: number;
  pageNo: number;
  regionId: string;
  start: number;
  text: string;
}

const TEXT_TAGS = [
  'blockquote',
  'del',
  'em',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'li',
  'p',
  'strong',
  'td',
  'th',
] as const;

const SKIPPED_ELEMENT_TYPES = new Set(['code', 'pre']);

export function TraceableMarkdown({
  onRegionSelect,
  page,
  selectedRegionId,
}: TraceableMarkdownProps) {
  const candidates = useMemo(() => traceCandidates(page), [page]);
  const components = useMemo(
    () => createTraceComponents(candidates, selectedRegionId, onRegionSelect),
    [candidates, onRegionSelect, selectedRegionId],
  );
  return (
    <div className={styles.markdownBody}>
      <ReactMarkdown components={components} remarkPlugins={[remarkGfm]}>
        {page.text}
      </ReactMarkdown>
    </div>
  );
}

export function PlainTraceText({ onRegionSelect, page, selectedRegionId }: TraceableMarkdownProps) {
  return <p>{renderPlainTraceText(page, selectedRegionId, onRegionSelect)}</p>;
}

function createTraceComponents(
  candidates: TraceCandidate[],
  selectedRegionId: string | undefined,
  onRegionSelect: (pageNo: number, regionId: string) => void,
): Components {
  const components: Record<string, (props: MarkdownComponentProps) => ReactNode> = {};
  for (const tag of TEXT_TAGS) {
    components[tag] = ({ children, node: _node, ...props }) =>
      createElement(
        tag,
        props,
        traceReactNode(children, candidates, selectedRegionId, onRegionSelect),
      );
  }
  return {
    ...components,
    a: ({ children, href, node: _node, ...props }) => {
      const matching = exactCandidateFromChildren(children, candidates);
      if (matching) {
        return traceButton(matching, textFromReactNode(children), selectedRegionId, onRegionSelect);
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

type MarkdownComponentProps = {
  children?: ReactNode;
  href?: string;
  node?: unknown;
  [key: string]: unknown;
};

function traceReactNode(
  node: ReactNode,
  candidates: TraceCandidate[],
  selectedRegionId: string | undefined,
  onRegionSelect: (pageNo: number, regionId: string) => void,
): ReactNode {
  if (node === null || node === undefined || typeof node === 'boolean') {
    return node;
  }
  if (typeof node === 'string') {
    return traceTextNode(node, candidates, selectedRegionId, onRegionSelect);
  }
  if (Array.isArray(node)) {
    return Children.map(node, (child) =>
      traceReactNode(child, candidates, selectedRegionId, onRegionSelect),
    );
  }
  if (!isValidElement(node)) {
    return node;
  }
  if (typeof node.type === 'string' && SKIPPED_ELEMENT_TYPES.has(node.type)) {
    return node;
  }
  const element = node as ReactElement<{ children?: ReactNode }>;
  if (element.props.children === undefined) {
    return node;
  }
  return cloneElement(
    element,
    undefined,
    traceReactNode(element.props.children, candidates, selectedRegionId, onRegionSelect),
  );
}

function traceTextNode(
  text: string,
  candidates: TraceCandidate[],
  selectedRegionId: string | undefined,
  onRegionSelect: (pageNo: number, regionId: string) => void,
) {
  const matches = findCandidateMatches(text, candidates);
  if (matches.length === 0) {
    return text;
  }
  const nodes: ReactNode[] = [];
  let cursor = 0;
  for (const match of matches) {
    if (match.start > cursor) {
      nodes.push(text.slice(cursor, match.start));
    }
    nodes.push(
      traceButton(
        match.candidate,
        text.slice(match.start, match.end),
        selectedRegionId,
        onRegionSelect,
        `${match.candidate.regionId}-${match.start}`,
      ),
    );
    cursor = match.end;
  }
  if (cursor < text.length) {
    nodes.push(text.slice(cursor));
  }
  return nodes;
}

function renderPlainTraceText(
  page: PageTextRecord,
  selectedRegionId: string | undefined,
  onRegionSelect: (pageNo: number, regionId: string) => void,
) {
  if (page.spans.length === 0) {
    return page.text;
  }
  const nodes: ReactNode[] = [];
  let cursor = 0;
  for (const span of sortedSpans(page.spans)) {
    if (span.start > cursor) {
      nodes.push(page.text.slice(cursor, span.start));
    }
    nodes.push(
      traceButton(
        spanCandidate(page, span),
        page.text.slice(span.start, span.end),
        selectedRegionId,
        onRegionSelect,
      ),
    );
    cursor = Math.max(cursor, span.end);
  }
  if (cursor < page.text.length) {
    nodes.push(page.text.slice(cursor));
  }
  return nodes;
}

function traceButton(
  candidate: TraceCandidate,
  text: string,
  selectedRegionId: string | undefined,
  onRegionSelect: (pageNo: number, regionId: string) => void,
  key?: string,
) {
  return (
    <button
      className={styles.span}
      data-active={selectedRegionId === candidate.regionId}
      data-region-id={candidate.regionId}
      key={key ?? `${candidate.regionId}-${candidate.start}`}
      onClick={() => onRegionSelect(candidate.pageNo, candidate.regionId)}
      type="button"
    >
      {text}
    </button>
  );
}

function traceCandidates(page: PageTextRecord): TraceCandidate[] {
  const unique = new Map<string, TraceCandidate>();
  for (const span of sortedSpans(page.spans)) {
    const candidate = spanCandidate(page, span);
    if (candidate.text.length > 0 && !unique.has(candidate.regionId)) {
      unique.set(candidate.regionId, candidate);
    }
  }
  return [...unique.values()].sort((left, right) => right.text.length - left.text.length);
}

function spanCandidate(page: PageTextRecord, span: TextRegionSpan): TraceCandidate {
  return {
    end: span.end,
    pageNo: span.page_no,
    regionId: span.region_id,
    start: span.start,
    text: page.text.slice(span.start, span.end).trim(),
  };
}

function sortedSpans(spans: TextRegionSpan[]) {
  return [...spans].sort((left, right) => left.start - right.start || left.end - right.end);
}

function findCandidateMatches(text: string, candidates: TraceCandidate[]) {
  const matches: Array<{ candidate: TraceCandidate; end: number; start: number }> = [];
  for (const candidate of candidates) {
    let start = text.indexOf(candidate.text);
    while (start >= 0) {
      const end = start + candidate.text.length;
      if (!matches.some((match) => start < match.end && end > match.start)) {
        matches.push({ candidate, end, start });
      }
      start = text.indexOf(candidate.text, end);
    }
  }
  return matches.sort((left, right) => left.start - right.start || right.end - left.end);
}

function exactCandidateFromChildren(children: ReactNode, candidates: TraceCandidate[]) {
  const text = textFromReactNode(children).trim();
  return candidates.find((candidate) => candidate.text === text);
}

function textFromReactNode(node: ReactNode): string {
  if (typeof node === 'string' || typeof node === 'number') {
    return String(node);
  }
  if (Array.isArray(node)) {
    return node.map(textFromReactNode).join('');
  }
  if (isValidElement(node)) {
    const element = node as ReactElement<{ children?: ReactNode }>;
    return textFromReactNode(element.props.children);
  }
  return '';
}
