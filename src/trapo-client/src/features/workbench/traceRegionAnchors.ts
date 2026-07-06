import { annotationIdOf } from '../../api/annotationIdentity';
import type { OverlayBox, PageTextRecord, TextRegionSpan } from '../../api/types';

export interface RegionAnchor {
  pageNo: number;
  regionId: string;
  start: number;
}

export interface IndexedRegionAnchor extends RegionAnchor {
  index: number;
}

export const REGION_MARKER = '#';
const REGION_SCHEME = '#trapo-region=';

export function regionAnchors(page: PageTextRecord): RegionAnchor[] {
  const byteLength = utf8ByteLength(page.text);
  const unique = new Map<string, RegionAnchor>();
  for (const span of sortedSpans(page.spans)) {
    const annotationId = annotationIdOf(span);
    if (span.start <= byteLength && !unique.has(annotationId)) {
      unique.set(annotationId, spanAnchor(span));
    }
  }
  return [...unique.values()];
}

export function indexedRegionAnchors(text: string, anchors: RegionAnchor[]): IndexedRegionAnchor[] {
  return anchors
    .map((anchor) => ({
      ...anchor,
      index: byteOffsetToStringIndex(text, anchor.start),
    }))
    .sort((left, right) => left.index - right.index || left.regionId.localeCompare(right.regionId));
}

export function overlayRegionMap(regions: OverlayBox[]) {
  return new Map(regions.map((region) => [annotationIdOf(region), region]));
}

export function regionIdFromHref(href?: string) {
  if (!href?.startsWith(REGION_SCHEME)) {
    return undefined;
  }
  return decodeURIComponent(href.slice(REGION_SCHEME.length));
}

export function snippetFromRegion(region?: OverlayBox) {
  if (region?.content_html !== 'image-snippet') {
    return undefined;
  }
  const match = /^!\[([^\]]*)\]\(([^)]+)\)$/.exec(region.content_markdown ?? '');
  if (!match) {
    return undefined;
  }
  return { alt: match[1], src: match[2] };
}

export function scopedRegionText(pages: PageTextRecord[], regionId: string) {
  for (const page of pages) {
    const span = page.spans.find((item) => annotationIdOf(item) === regionId);
    if (!span || span.start > utf8ByteLength(page.text)) {
      continue;
    }
    const start = byteOffsetToStringIndex(page.text, span.start);
    const next = nextSpanStart(page.spans, span.start);
    const end = next ? byteOffsetToStringIndex(page.text, next) : page.text.length;
    return page.text.slice(start, end).trim();
  }
  return undefined;
}

function nextSpanStart(spans: TextRegionSpan[], currentStart: number) {
  return spans
    .map((item) => item.start)
    .filter((start) => start > currentStart)
    .sort((left, right) => left - right)[0];
}

function spanAnchor(span: TextRegionSpan): RegionAnchor {
  return {
    pageNo: span.page_no,
    regionId: annotationIdOf(span),
    start: span.start,
  };
}

function sortedSpans(spans: TextRegionSpan[]) {
  return [...spans].sort((left, right) => left.start - right.start || left.end - right.end);
}

function utf8ByteLength(text: string) {
  let bytes = 0;
  for (let index = 0; index < text.length; ) {
    const codePoint = text.codePointAt(index) ?? 0;
    bytes += utf8CodePointLength(codePoint);
    index += codePoint > 0xffff ? 2 : 1;
  }
  return bytes;
}

function byteOffsetToStringIndex(text: string, byteOffset: number) {
  let bytes = 0;
  for (let index = 0; index < text.length; ) {
    if (bytes >= byteOffset) {
      return index;
    }
    const codePoint = text.codePointAt(index) ?? 0;
    const nextBytes = bytes + utf8CodePointLength(codePoint);
    if (nextBytes > byteOffset) {
      return index;
    }
    bytes = nextBytes;
    index += codePoint > 0xffff ? 2 : 1;
  }
  return text.length;
}

function utf8CodePointLength(codePoint: number) {
  if (codePoint <= 0x7f) {
    return 1;
  }
  if (codePoint <= 0x7ff) {
    return 2;
  }
  if (codePoint <= 0xffff) {
    return 3;
  }
  return 4;
}
