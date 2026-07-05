# Trapo Agent Notes

## Core Client Requirements

- The Download Manager is root-level UI for the entire React app. Do not make it
  local to `/models`, `/models/downloads`, or any other feature route.
- The global `downloads=true` query parameter is the source of truth for whether
  the downloads pane is visible. Preserve it across all TanStack Router
  navigations.
- The status bar must keep an always-visible downloads toggle on every Trapo
  page.
- Starting a model download or re-download must open the downloads pane by
  default so the user can track progress immediately.

## Core Persistence Requirements

- Every generated persistence identity, primary key, stream event id, diagnostic
  id, model download id, and annotation id must be a UUID v7 string. Do not add
  ULIDs, timestamp ids, geometry hashes, or deterministic strings as generated
  primary keys. Deterministic keys are allowed only as secondary source keys for
  deduplication or lookup.
- OCR annotation regions get a persisted UUID v7 `annotation_id` as soon as the
  bounding box is discovered. Text can arrive later, but all future text spans,
  snippet records, overlay boxes, realtime events, and persisted rows for that
  region must refer back to the same annotation id.
- The React workbench must use `annotation_id` as the cross-pane identity for
  text-preview anchors, image-snippet anchors, PDF/image overlay boxes, selected
  region state, DOM ids, and `data-annotation-id` attributes. Keep `region_id`
  fallback support only for older payloads and fixtures.
