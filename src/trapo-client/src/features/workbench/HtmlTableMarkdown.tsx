import type { Components } from 'react-markdown';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

import styles from './TextPane.module.css';

interface HtmlTableCell {
  header: boolean;
  key: string;
  text: string;
}

interface HtmlTableRow {
  cells: HtmlTableCell[];
  key: string;
}

type HtmlTableBlock =
  | { key: string; kind: 'markdown'; value: string }
  | { key: string; kind: 'table'; value: string };

export function MarkdownWithHtmlTables({
  components,
  markdown,
}: {
  components: Components;
  markdown: string;
}) {
  return (
    <>
      {splitHtmlTables(markdown).map((block) =>
        block.kind === 'table' ? (
          <HtmlTable components={components} html={block.value} key={block.key} />
        ) : (
          <ReactMarkdown components={components} key={block.key} remarkPlugins={[remarkGfm]}>
            {block.value}
          </ReactMarkdown>
        ),
      )}
    </>
  );
}

function HtmlTable({ components, html }: { components: Components; html: string }) {
  const rows = parseHtmlTable(html);
  if (rows.length === 0) {
    return <p>{decodeHtml(stripTags(html))}</p>;
  }
  return (
    <div className={styles.tableViewport}>
      <table>
        <tbody>
          {rows.map((row) => (
            <tr key={row.key}>
              {row.cells.map((cell) => {
                const Cell = cell.header ? 'th' : 'td';
                return (
                  <Cell key={cell.key}>
                    <ReactMarkdown components={components} remarkPlugins={[remarkGfm]}>
                      {cell.text}
                    </ReactMarkdown>
                  </Cell>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function splitHtmlTables(markdown: string): HtmlTableBlock[] {
  const blocks: HtmlTableBlock[] = [];
  const tablePattern = /<table\b[\s\S]*?<\/table>/gi;
  let cursor = 0;
  for (const match of markdown.matchAll(tablePattern)) {
    const start = match.index ?? 0;
    if (start > cursor) {
      blocks.push(block('markdown', markdown.slice(cursor, start), blocks.length));
    }
    blocks.push(block('table', match[0], blocks.length));
    cursor = start + match[0].length;
  }
  if (cursor < markdown.length) {
    blocks.push(block('markdown', markdown.slice(cursor), blocks.length));
  }
  return blocks.length > 0 ? blocks : [block('markdown', markdown, 0)];
}

export function parseHtmlTable(html: string): HtmlTableRow[] {
  const rows: HtmlTableRow[] = [];
  const rowPattern = /<tr\b[^>]*>([\s\S]*?)<\/tr>/gi;
  for (const rowMatch of html.matchAll(rowPattern)) {
    const rowIndex = rows.length;
    const cells: HtmlTableCell[] = [];
    const cellPattern = /<(td|th)\b[^>]*>([\s\S]*?)<\/\1>/gi;
    for (const cellMatch of (rowMatch[1] ?? '').matchAll(cellPattern)) {
      const cellIndex = cells.length;
      const tag = cellMatch[1] ?? 'td';
      const value = cellMatch[2] ?? '';
      cells.push({
        header: tag.toLowerCase() === 'th',
        key: `cell-${rowIndex}-${cellIndex}-${stableKey(cellMatch[0])}`,
        text: decodeHtml(stripTags(value)).trim(),
      });
    }
    if (cells.length > 0) {
      rows.push({
        cells,
        key: `row-${rowIndex}-${stableKey(rowMatch[0])}`,
      });
    }
  }
  return rows;
}

function block(kind: HtmlTableBlock['kind'], value: string, index: number): HtmlTableBlock {
  return { key: `${kind}-${index}-${stableKey(value)}`, kind, value };
}

function stableKey(value: string) {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(36);
}

function stripTags(value: string) {
  return value.replace(/<br\s*\/?>/gi, '\n').replace(/<[^>]+>/g, '');
}

function decodeHtml(value: string) {
  return value
    .replace(/&#x([0-9a-f]+);/gi, (_match, code: string) =>
      String.fromCodePoint(Number.parseInt(code, 16)),
    )
    .replace(/&#(\d+);/g, (_match, code: string) => String.fromCodePoint(Number.parseInt(code, 10)))
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&amp;/g, '&');
}
