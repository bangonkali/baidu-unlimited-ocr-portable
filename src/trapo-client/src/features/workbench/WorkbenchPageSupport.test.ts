import { describe, expect, test } from 'bun:test';

import type { DocumentRegionsPayload } from '../../api/types';
import { shouldFollowLatestRegion } from './WorkbenchPageSupport';

describe('shouldFollowLatestRegion', () => {
  test('does not follow a stale region from an earlier page during page transition', () => {
    expect(
      shouldFollowLatestRegion(
        { fileHash: 'live-doc', pageNo: 2, regionId: undefined },
        regionsPayload(1, 'page-1-region'),
      ),
    ).toBe(false);
  });

  test('follows a new region on the selected or later page', () => {
    expect(
      shouldFollowLatestRegion(
        { fileHash: 'live-doc', pageNo: 2, regionId: undefined },
        regionsPayload(2, 'page-2-region'),
      ),
    ).toBe(true);
  });

  test('does not refollow the already selected region', () => {
    expect(
      shouldFollowLatestRegion(
        { fileHash: 'live-doc', pageNo: 2, regionId: 'page-2-region' },
        regionsPayload(2, 'page-2-region'),
      ),
    ).toBe(false);
  });

  test('does not follow regions from another run for the same file', () => {
    expect(
      shouldFollowLatestRegion(
        { fileHash: 'live-doc', pageNo: 1, regionId: undefined, runId: 'run-a' },
        regionsPayload(1, 'page-1-region', 'run-b'),
      ),
    ).toBe(false);
  });
});

function regionsPayload(pageNo: number, regionId: string, runId = 'run-a'): DocumentRegionsPayload {
  return {
    boxes: [
      {
        height_percent: 10,
        label: 'Region',
        left_percent: 10,
        page_no: pageNo,
        region_id: regionId,
        top_percent: 10,
        width_percent: 10,
      },
    ],
    file_hash: 'live-doc',
    run_id: runId,
  };
}
