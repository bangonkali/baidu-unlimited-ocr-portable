# Text Preview

The Trapo text preview is a live view of OCR output for the selected document.
It is intentionally not a raw dump of every rendered page while ingest is still
running.

## Visibility Contract

During an active ingest, the text preview shows:

- pages that have completed OCR
- the page currently running OCR
- pages with live streamed text or spans

Queued future pages are hidden. This prevents auto-follow from jumping to an
empty placeholder at the end of a large PDF when the current page is still
streaming.

After a document reaches a terminal state, Trapo can show all pages, including
blank pages that completed OCR without text.

## Backend Behavior

The Rust server marks a page `running` when OCR starts. `/api/documents/{file_hash}/text`
and `document.text.changed` payloads include only pages whose status is no
longer `queued`.

Detected regions are stored as zero-width text anchors at the OCR marker
location. A region's text scope starts at that anchor and ends at the next
region anchor, or at the end of the page text. This keeps bounding boxes,
details-pane content, and text-preview navigation aligned even when the OCR text
around a marker changes.

Live OCR emits incremental events while a page runs:

```text
ocr.page.stream.started
ocr.page.text.patch
ocr.page.region.upsert
ocr.page.span.upsert
ocr.page.stream.completed
```

Replayable `ocr.page.*` events are inserted into DuckDB before websocket
broadcast. The client can request those events from `/api/ocr/events` and apply
them through the same reducer primitives used for live websocket updates. Replay
hydration is projected from history before it is merged into the cache, so
repeated refreshes or replay polling do not append the same streamed text twice.
This lets the Workbench rebuild the selected page after a refresh while OCR is
still running. After completion, final document/page/region tables are the
source of truth.

## Client Behavior

The React workbench applies one more defensive filter before rendering the text
pane. If an older payload contains queued placeholder pages, the client derives
visible pages from document progress, `current_page`, existing text, and spans.

When a new ingest starts, the client seeds the selected file and page from the
`/api/ingest/start` response before navigating to `/workbench`. The selected
active page replays persisted OCR events while it is still running. If the
websocket reconnects with a sequence gap, the client reads missed `ocr.page.*`
events from DuckDB and invalidates table-backed run, document, status, and log
queries.

Region anchors render as compact `#` controls instead of wrapping a text span.
Clicking a PDF annotation focuses the matching text anchor. Clicking a text
anchor focuses the matching PDF annotation.

Auto-follow scrolls to the currently active text anchor or region instead of the
last page placeholder.

## Rich Preview Content

OCR output can contain Markdown tables, raw HTML tables, and image-like regions.
Trapo renders Markdown tables through GitHub-flavored Markdown. Raw OCR
`<table>` markup is parsed into safe React table elements instead of being
inserted as raw HTML.

When a detected region looks image-like, the server crops the matching area from
the rendered page image and writes a PNG snippet under the Trapo cache. The text
preview embeds that snippet next to the region anchor using:

```text
/api/documents/{file_hash}/regions/{region_id}/snippet
```

The snippet URL is deterministic and local to the running Trapo server. Future
image introspection can attach richer metadata to the same region anchor without
changing the preview navigation model.
