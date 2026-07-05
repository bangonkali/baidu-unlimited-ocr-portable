import { describe, expect, test } from 'bun:test';

import { parseHtmlTable, splitHtmlTables } from './HtmlTableMarkdown';

describe('HtmlTableMarkdown', () => {
  test('uses unique keys for repeated empty and identical table cells', () => {
    const rows = parseHtmlTable(
      '<table><tr><td></td><td></td><td>Total</td><td>Total</td></tr></table>',
    );
    const keys = rows.flatMap((row) => row.cells.map((cell) => cell.key));

    expect(keys).toHaveLength(4);
    expect(new Set(keys).size).toBe(4);
  });

  test('uses unique keys for repeated identical html table blocks', () => {
    const blocks = splitHtmlTables(
      '<table><tr><td></td></tr></table><table><tr><td></td></tr></table>',
    );
    const keys = blocks.map((block) => block.key);

    expect(keys).toHaveLength(2);
    expect(new Set(keys).size).toBe(2);
  });
});
