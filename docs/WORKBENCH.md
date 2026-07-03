# Workbench Flow

The Trapo Workbench is the local document inspection surface served by
`trapo-server` from the compiled React app in `src/trapo-client`. It is designed
to feel compact and operational, with a VS Code inspired activity bar,
tree-style explorer rows, lower-right notifications, and persistent panels.

## Start OCR Entry

The Workbench no longer uses a top toolbar. The primary entry point is the Start
OCR icon in the activity bar.

- If the selected model is already downloaded, the icon opens `/ingest/start`.
- If the selected model is missing locally, the icon opens `/models` and writes
  a notification to the lower-right notification history.
- The notification bell keeps historical messages visible even after routing.

The dedicated ingest page owns folder selection and run creation. It sends an
`IngestStartRequest` to `trapo-server`, which discovers supported files and
creates an ingest run in DuckDB before background processing begins.

The start response is a causal snapshot, not just an acknowledgement. It returns
the accepted run, ordered file hashes, discovered document summaries, and the
OCR replay sequence to resume from. The React client seeds the run and document
caches from that response, turns auto-follow on for the new run, selects the
first discovered document, and then navigates to `/workbench` with explicit
`file`, `page`, and `follow` route state. The preview Auto Follow button updates
the route's `follow` value as well as the local store, so manual focus links can
turn following off without preventing the user from turning it back on. This
keeps the preview and text panes reliable even when the websocket connects
during the route transition.

## File Explorer

The explorer follows the selected target directory structure. Every visible row
is a tree node with compact height, a twisty where needed, and an icon that
represents the current state:

- queued work uses a clock icon
- active OCR uses a spinner icon
- completed work uses a check icon
- failed work uses an alert icon

Rows are built with the shared `TreeView` and `TreeGrid` components under
`src/trapo-client/src/components/workbench`. Feature panels pass nodes into that
shared component instead of hand-rolling separate tree renderers.

Page rows are generated from numeric document `page_count` values and carry a
numeric `pageNo` sort key. The explorer displays labels such as `Page 10`, but
sorting never parses those labels as strings; page children are ordered by the
actual page number so multi-page files appear as `Page 1`, `Page 2`, and then
later `Page 10`.

## Download Manager

Downloads are coordinated by `trapo-server` as a serial file queue. Model
downloads enqueue their required GGUF files into the same generic queue used by
future file download operations. The model manager starts, cancels, re-downloads,
or selects a model, while the separate Download Manager shows only queued or
in-progress files.

`/models/downloads` is the active-downloads route. It opens the Models surface in
a queue-focused scope and automatically opens the Download Manager. Missing,
failed, cancelled, or already downloaded files stay in the main `/models`
library, similar to Chrome's split between the active download tray and the full
downloads list.

Download start, completion, cancellation, and failure events are recorded in
DuckDB `download_events`. Runtime status still comes from the filesystem first:
if a GGUF file was previously downloaded but is no longer present under
`models/`, the model appears as a neutral missing state and offers download
again instead of trusting stale history.

## Onboarding And Tooltips

React Joyride guides the user through the Start OCR icon, model downloader,
ingest page, explorer, preview, text pane, and diagnostics route. Icon-heavy
controls should expose accessible labels and concise tooltips so the UI stays
compact without hiding meaning from keyboard and screen-reader users.
