import { Throttler } from '@tanstack/react-pacer';

import { annotationIdOf } from '../api/annotationIdentity';
import type { OverlayBox } from '../api/types';
import type { WorkbenchState } from './workbenchStore';

export const OCR_FOCUS_THROTTLE_MS = 1000;

type RealtimeFocusTarget =
  | {
      fileHash: string;
      kind: 'page';
      pageNo: number;
      runEngineId?: string | null;
      runId?: string | null;
    }
  | {
      fileHash: string;
      kind: 'region';
      pageNo: number;
      regionId: string;
      runEngineId?: string | null;
      runId?: string | null;
    };

export function createRealtimeFocusScheduler(applyFocus: (target: RealtimeFocusTarget) => void) {
  const throttler = new Throttler(applyFocus, {
    key: 'workbench-ocr-focus',
    leading: true,
    trailing: true,
    wait: OCR_FOCUS_THROTTLE_MS,
  });

  return {
    cancel: () => throttler.cancel(),
    flush: () => throttler.flush(),
    reset: () => {
      throttler.cancel();
      throttler.reset();
    },
    schedule: (target: RealtimeFocusTarget) => throttler.maybeExecute(target),
  };
}

export function pageFocusTarget(
  fileHash: string,
  pageNo: number,
  runId?: string | null,
  runEngineId?: string | null,
): RealtimeFocusTarget {
  return { fileHash, kind: 'page', pageNo, runEngineId, runId };
}

export function latestRegionFocusTarget(
  fileHash: string,
  boxes: OverlayBox[],
  runId?: string | null,
  runEngineId?: string | null,
): RealtimeFocusTarget | undefined {
  const latest = boxes.at(-1);
  return latest
    ? {
        fileHash,
        kind: 'region',
        pageNo: latest.page_no,
        regionId: annotationIdOf(latest),
        runEngineId,
        runId,
      }
    : undefined;
}

export function stateAfterRealtimeFocus(
  state: WorkbenchState,
  target: RealtimeFocusTarget,
): WorkbenchState {
  if (!state.autoFollowRegions) {
    return state;
  }
  const selection =
    target.kind === 'region'
      ? {
          ...state.selection,
          fileHash: target.fileHash,
          pageNo: target.pageNo,
          regionId: target.regionId,
          runEngineId: target.runEngineId ?? state.selection.runEngineId,
          runId: target.runId ?? state.selection.runId,
        }
      : pageSelection(state, target);
  return state.activeView === 'workbench' && sameSelection(state.selection, selection)
    ? state
    : { ...state, activeView: 'workbench', selection, selectionSource: 'realtime' };
}

function pageSelection(
  state: WorkbenchState,
  target: Extract<RealtimeFocusTarget, { kind: 'page' }>,
) {
  const nextRunId = target.runId ?? state.selection.runId;
  const nextRunEngineId = target.runEngineId ?? state.selection.runEngineId;
  const pageChanged =
    state.selection.fileHash !== target.fileHash || // skylos: ignore[SKY-D253] fileHash is public workbench route state, not a secret token.
    state.selection.pageNo !== target.pageNo ||
    state.selection.runEngineId !== nextRunEngineId ||
    state.selection.runId !== nextRunId;
  return {
    ...state.selection,
    fileHash: target.fileHash,
    pageNo: target.pageNo,
    regionId: pageChanged ? undefined : state.selection.regionId,
    runEngineId: nextRunEngineId,
    runId: nextRunId,
  };
}

function sameSelection(left: WorkbenchState['selection'], right: WorkbenchState['selection']) {
  return (
    left.fileHash === right.fileHash && // skylos: ignore[SKY-D253] fileHash is public workbench route state, not a secret token.
    left.pageNo === right.pageNo &&
    left.regionId === right.regionId &&
    left.runEngineId === right.runEngineId &&
    left.runId === right.runId
  );
}
