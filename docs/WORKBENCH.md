# Workbench Flow

The Trapo Workbench is the local document inspection surface served by
`trapo-server` from the compiled React app in `src/trapo-client`. It is designed
to feel compact and operational, with a VS Code inspired activity bar,
tree-style explorer rows, lower-right notifications, and persistent panels.

## Start OCR Entry

The Workbench no longer uses a top toolbar. The primary entry point is the Start
OCR icon in the activity bar.

- If the selected model is already downloaded, the icon opens `/ingest/start`.
- If no selected model is downloaded, the icon opens `/models/downloads` and
  writes a notification to the lower-right notification history.
- The notification bell keeps historical messages visible even after routing.

The dedicated ingest page owns folder selection and run creation. It sends an
`IngestStartRequest` to `trapo-server`, which discovers supported files and
creates an ingest run in DuckDB before background processing begins.

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

## Download Manager

Model downloads are coordinated by `trapo-server` as a serial queue. The model
manager starts, cancels, deletes, or selects a model, while the separate
Download Manager shows queue-level and file-level progress.

The manager can stay open while the user navigates across pages. It shows the
active model, queued models, downloaded bytes, total bytes when known, estimated
time remaining, current file, and cancel controls. Model cards and grids keep
their own UI compact by showing only model-specific state and actions.

## Onboarding And Tooltips

React Joyride guides the user through the Start OCR icon, model downloader,
ingest page, explorer, preview, text pane, and diagnostics route. Icon-heavy
controls should expose accessible labels and concise tooltips so the UI stays
compact without hiding meaning from keyboard and screen-reader users.
