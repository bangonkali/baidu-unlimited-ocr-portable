import type { ReactNode } from 'react';

import type { PageTextRecord } from '../../api/types';
import { setSelection } from '../../stores/workbenchStore';
import styles from './TextPane.module.css';

interface TextPaneProps {
  pages: PageTextRecord[];
  selectedRegionId?: string;
}

export function TextPane({ pages, selectedRegionId }: TextPaneProps) {
  return (
    <section className={styles.textPane} aria-label="Text">
      <div className={styles.header}>Text</div>
      <div className={styles.body}>
        {pages.length === 0 ? <div className={styles.empty}>No OCR text</div> : null}
        {pages.map((page) => (
          <article className={styles.pageText} key={page.page_no}>
            <div className={styles.pageLabel}>Page {page.page_no}</div>
            <p>{renderPageText(page, selectedRegionId)}</p>
          </article>
        ))}
      </div>
    </section>
  );
}

function renderPageText(page: PageTextRecord, selectedRegionId?: string) {
  if (page.spans.length === 0) {
    return page.text;
  }
  const nodes: ReactNode[] = [];
  let cursor = 0;
  for (const span of page.spans) {
    if (span.start > cursor) {
      nodes.push(page.text.slice(cursor, span.start));
    }
    nodes.push(
      <button
        className={styles.span}
        data-active={selectedRegionId === span.region_id}
        key={`${span.region_id}-${span.start}`}
        onClick={() => setSelection({ pageNo: page.page_no, regionId: span.region_id })}
        type="button"
      >
        {page.text.slice(span.start, span.end)}
      </button>,
    );
    cursor = span.end;
  }
  if (cursor < page.text.length) {
    nodes.push(page.text.slice(cursor));
  }
  return nodes;
}
