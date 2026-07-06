import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { useThrottledValue } from '@tanstack/react-pacer';
import { ChevronDown, Copy } from 'lucide-react';
import type { RefObject } from 'react';
import { useEffect, useMemo, useRef, useState } from 'react';

import { annotationTextDomId } from '../../api/annotationIdentity';
import type { DocumentSummary, OverlayBox, PageTextRecord } from '../../api/types';
import { OCR_FOCUS_THROTTLE_MS } from '../../stores/workbenchRealtimeFocus';
import { isScrolledToBottom, needsRevealScroll } from './scrollVisibility';
import styles from './TextPane.module.css';
import { PlainTraceText, TraceableMarkdown } from './TraceableMarkdown';
import { fileMarkdown, filePlainText, isPageTextComplete, pageMarkdown } from './textExport';

interface TextPaneProps {
  autoFollowRegions: boolean;
  document?: DocumentSummary;
  focusRevision?: number;
  pages: PageTextRecord[];
  regions: OverlayBox[];
  selectedRegionId?: string;
  onSelectRegion: (pageNo: number, regionId: string) => void;
}

export function TextPane({
  autoFollowRegions,
  document,
  focusRevision = 0,
  onSelectRegion,
  pages,
  regions,
  selectedRegionId,
}: TextPaneProps) {
  const bodyRef = useRef<HTMLDivElement>(null);
  const [copyStatus, setCopyStatus] = useState('');
  const glowRegionId = useRegionFocusGlow(selectedRegionId, focusRevision);
  const fingerprint = useMemo(
    () => pages.map((page) => `${page.page_no}:${page.text.length}:${page.spans.length}`).join('|'),
    [pages],
  );
  const [scrollFingerprint] = useThrottledValue(fingerprint, {
    key: 'workbench-text-live-scroll',
    leading: true,
    trailing: true,
    wait: OCR_FOCUS_THROTTLE_MS,
  });

  useTextAutoScroll(bodyRef, autoFollowRegions, selectedRegionId, scrollFingerprint, focusRevision);

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
            glowRegionId={glowRegionId}
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
  glowRegionId,
  selectedRegionId,
}: {
  complete: boolean;
  glowRegionId?: string;
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
          glowRegionId={glowRegionId}
          selectedRegionId={selectedRegionId}
        />
      ) : (
        <PlainTraceText
          onRegionSelect={onRegionSelect}
          page={page}
          regions={regions}
          glowRegionId={glowRegionId}
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
  focusRevision: number,
) {
  useEffect(() => {
    const root = bodyRef.current;
    if (!root) {
      return;
    }
    const revealGeneration = focusRevision;
    if (selectedRegionId && revealGeneration >= 0) {
      const selected = findAnnotationTextElement(root, selectedRegionId);
      if (!selected) {
        return;
      }
      if (needsRevealScroll(root.getBoundingClientRect(), selected.getBoundingClientRect())) {
        selected.scrollIntoView({ behavior: 'smooth', block: 'center', inline: 'nearest' });
      }
      return;
    }
    if (!autoFollowRegions || !fingerprint) {
      return;
    }
    if (isScrolledToBottom(root)) {
      return;
    }
    root.scrollTo({ behavior: 'smooth', top: root.scrollHeight });
  }, [autoFollowRegions, bodyRef, fingerprint, focusRevision, selectedRegionId]);
}

function useRegionFocusGlow(selectedRegionId: string | undefined, focusRevision: number) {
  const [glowRegionId, setGlowRegionId] = useState<string | undefined>(undefined);
  useEffect(() => {
    if (!selectedRegionId || focusRevision <= 0) {
      setGlowRegionId(undefined);
      return;
    }
    setGlowRegionId(selectedRegionId);
    const timeoutId = window.setTimeout(() => setGlowRegionId(undefined), 1600);
    return () => window.clearTimeout(timeoutId);
  }, [focusRevision, selectedRegionId]);
  return glowRegionId;
}

export function findAnnotationTextElement(root: HTMLElement, annotationId: string) {
  const element = root.ownerDocument.getElementById(annotationTextDomId(annotationId));
  return element instanceof HTMLElement && root.contains(element) ? element : null;
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
