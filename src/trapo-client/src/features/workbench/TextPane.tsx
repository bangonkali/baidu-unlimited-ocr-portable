import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { ChevronDown, Copy } from 'lucide-react';
import type { RefObject } from 'react';
import { useEffect, useMemo, useRef, useState } from 'react';

import type { DocumentSummary, OverlayBox, PageTextRecord } from '../../api/types';
import styles from './TextPane.module.css';
import { PlainTraceText, TraceableMarkdown } from './TraceableMarkdown';
import { fileMarkdown, filePlainText, isPageTextComplete, pageMarkdown } from './textExport';

interface TextPaneProps {
  autoFollowRegions: boolean;
  document?: DocumentSummary;
  pages: PageTextRecord[];
  regions: OverlayBox[];
  selectedRegionId?: string;
  onSelectRegion: (pageNo: number, regionId: string) => void;
}

export function TextPane({
  autoFollowRegions,
  document,
  onSelectRegion,
  pages,
  regions,
  selectedRegionId,
}: TextPaneProps) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const [copyStatus, setCopyStatus] = useState('');
  const fingerprint = useMemo(
    () => pages.map((page) => `${page.page_no}:${page.text.length}:${page.spans.length}`).join('|'),
    [pages],
  );

  useTextAutoScroll(bodyRef, autoFollowRegions, selectedRegionId, fingerprint);

  const copy = (value: string, label: string) => {
    void copyToClipboard(value).then(() => {
      setCopyStatus(label);
      window.setTimeout(() => setCopyStatus(''), 1400);
    });
  };

  return (
    <section className={styles.textPane} aria-label="Text">
      <div className={styles.header}>
        <span>Text</span>
        <div className={styles.headerActions}>
          {copyStatus ? <span className={styles.copyStatus}>{copyStatus}</span> : null}
          <WholeFileCopyMenu
            disabled={pages.length === 0}
            onCopyMarkdown={() => copy(fileMarkdown(pages), 'Copied Markdown')}
            onCopyPlain={() => copy(filePlainText(pages), 'Copied text')}
          />
        </div>
      </div>
      <div className={styles.body} ref={bodyRef}>
        {pages.length === 0 ? <div className={styles.empty}>No OCR text</div> : null}
        {pages.map((page) => (
          <PageText
            complete={isPageTextComplete(page, document)}
            key={page.page_no}
            onCopyMarkdown={() => copy(pageMarkdown(page), `Copied page ${page.page_no}`)}
            onRegionSelect={onSelectRegion}
            page={page}
            regions={regions}
            selectedRegionId={selectedRegionId}
          />
        ))}
      </div>
    </section>
  );
}

function WholeFileCopyMenu({
  disabled,
  onCopyMarkdown,
  onCopyPlain,
}: {
  disabled: boolean;
  onCopyMarkdown: () => void;
  onCopyPlain: () => void;
}) {
  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button className={styles.headerIcon} disabled={disabled} title="Copy file" type="button">
          <Copy size={14} strokeWidth={1.9} />
          <ChevronDown size={12} strokeWidth={1.9} />
        </button>
      </DropdownMenu.Trigger>
      <DropdownMenu.Portal>
        <DropdownMenu.Content align="end" className={styles.copyMenu} sideOffset={4}>
          <DropdownMenu.Item className={styles.copyMenuItem} onSelect={onCopyMarkdown}>
            Copy Markdown
          </DropdownMenu.Item>
          <DropdownMenu.Item className={styles.copyMenuItem} onSelect={onCopyPlain}>
            Copy Plain Text
          </DropdownMenu.Item>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
}

function PageText({
  complete,
  onCopyMarkdown,
  onRegionSelect,
  page,
  regions,
  selectedRegionId,
}: {
  complete: boolean;
  onCopyMarkdown: () => void;
  onRegionSelect: (pageNo: number, regionId: string) => void;
  page: PageTextRecord;
  regions: OverlayBox[];
  selectedRegionId?: string;
}) {
  return (
    <article className={styles.pageText}>
      <div className={styles.pageHeader}>
        <span className={styles.pageLabel}>Page {page.page_no}</span>
        <button
          className={styles.pageCopy}
          disabled={page.text.trim().length === 0}
          onClick={onCopyMarkdown}
          title={`Copy page ${page.page_no}`}
          type="button"
        >
          <Copy size={13} strokeWidth={1.9} />
        </button>
      </div>
      {complete ? (
        <TraceableMarkdown
          onRegionSelect={onRegionSelect}
          page={page}
          regions={regions}
          selectedRegionId={selectedRegionId}
        />
      ) : (
        <PlainTraceText
          onRegionSelect={onRegionSelect}
          page={page}
          regions={regions}
          selectedRegionId={selectedRegionId}
        />
      )}
      {!complete ? <div className={styles.liveHint}>Live OCR text</div> : null}
    </article>
  );
}

function useTextAutoScroll(
  bodyRef: RefObject<HTMLDivElement | null>,
  autoFollowRegions: boolean,
  selectedRegionId: string | undefined,
  fingerprint: string,
) {
  useEffect(() => {
    if (!autoFollowRegions) {
      return;
    }
    const root = bodyRef.current;
    if (!root) {
      return;
    }
    if (!fingerprint && !selectedRegionId) {
      return;
    }
    const selected = selectedRegionId ? findTraceElement(root, selectedRegionId) : null;
    if (selected) {
      selected.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'nearest' });
      return;
    }
    root.scrollTo({ behavior: 'smooth', top: root.scrollHeight });
  }, [autoFollowRegions, bodyRef, fingerprint, selectedRegionId]);
}

function findTraceElement(root: HTMLElement, regionId: string) {
  return [...root.querySelectorAll<HTMLElement>('[data-annotation-id], [data-region-id]')].find(
    (element) => element.dataset.annotationId === regionId || element.dataset.regionId === regionId,
  );
}

async function copyToClipboard(value: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(value);
    return;
  }
  const textarea = document.createElement('textarea');
  textarea.value = value;
  textarea.style.position = 'fixed';
  textarea.style.opacity = '0';
  document.body.append(textarea);
  textarea.select();
  document.execCommand('copy');
  textarea.remove();
}
