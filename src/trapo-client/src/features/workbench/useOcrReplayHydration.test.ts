import { describe, expect, test } from 'bun:test';

import { selectedPageReplayRequest } from './useOcrReplayHydration';

describe('selected page OCR replay hydration', () => {
  test('builds a one-shot replay request without polling options', () => {
    const request = selectedPageReplayRequest({
      enabled: true,
      fileHash: 'file-a',
      pageNo: 3,
    });

    expect(request).toEqual({
      enabled: true,
      file_hash: 'file-a',
      limit: 10_000,
      page_no: 3,
    });
    expect('refetchInterval' in request).toBe(false);
  });
});
