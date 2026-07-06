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
`run`, `file`, `page`, and `follow` route state. The preview Auto Follow button
updates the route's `follow` value as well as the local store, so manual focus
links can turn following off without preventing the user from turning it back
on. This keeps the preview and text panes reliable even when the websocket
connects during the route transition.

## Run-Scoped OCR Results

OCR text, text spans, annotation ids, and overlay regions are scoped by ingest
run. The workbench URL must include `run=<run_id>` whenever it displays OCR
outputs, and document text/region API reads must pass the same value as
`run_id`. Preview images remain file/page assets, but overlays and text must
come only from the selected run.

This isolation is required because scanning the same file again creates a new
set of annotations. A fresh run must start with empty run-scoped text and
regions, then populate from websocket events and run-scoped DuckDB rows. It
must never show stale annotations from an older run for the same `file_hash`.

See [OCR Data Model](OCR-DATA-MODEL.md) for the canonical DuckDB and React
naming contract for folders, files, pages, runs, and annotations.

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

The Download Manager is root-level UI, not a child of the Models page. The
global `downloads=true` query parameter controls whether the pane is visible and
must be retained across every TanStack route. The status bar owns the always
visible toggle, so users can open or close download progress from `/workbench`,
`/ingest/start`, `/models`, `/settings`, `/diagnostics`, or any future page.

Starting a model download or re-download opens `?downloads=true` immediately so
progress is visible while the server queues files and realtime model updates
arrive. Realtime or refetched model data also opens the pane when the active file
count increases, which covers downloads started outside the current component.

`/models/downloads` remains the active-downloads route. It opens the Models
surface in a queue-focused scope and sets `downloads=true`, but it does not own
the pane. Missing, failed, cancelled, or already downloaded files stay in the
main `/models` library, similar to Chrome's split between the active download
tray and the full downloads list.

Download start, completion, cancellation, and failure events are recorded in
DuckDB `download_events`. Runtime status still comes from the filesystem first:
if a GGUF file was previously downloaded but is no longer present under
`models/`, the model appears as a neutral missing state and offers download
again instead of trusting stale history.

## Clean Shutdown

The status bar owns the always-visible Trapo shutdown control. The button opens
a confirmation prompt and then calls `POST /api/system/shutdown` with the
explicit shutdown confirmation body and intent header. This route is idempotent:
repeated clicks keep the same shutdown in progress instead of starting competing
shutdown flows.

Shutdown stops new ingest and model-download work, cancels active runs and
downloads, drains realtime and annotation persistence queues, checkpoints
DuckDB, flushes logs, and then lets the server process exit so native GPU and
database handles are released. Ctrl+C and supported OS console close/shutdown
signals use the same cleanup path.

The already-loaded React app switches to a friendly offline page after the
shutdown request or after status probes fail. A cold browser reload cannot be
served while the backend process is stopped; restart `trapo-server` first, then
use Retry or reload the browser.

## Onboarding And Tooltips

React Joyride guides the user through the Start OCR icon, model downloader,
ingest page, explorer, preview, text pane, and diagnostics route. Icon-heavy
controls should expose accessible labels and concise tooltips so the UI stays
compact without hiding meaning from keyboard and screen-reader users.
