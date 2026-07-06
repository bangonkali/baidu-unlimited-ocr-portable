# OCR Data Model

This document defines the shared names for folder, file, page, run, and
annotation data. Use these names in storage code, API contracts, React state,
tests, and documentation unless a legacy route already requires a different
parameter name.

## Canonical Shape

OCR data is a tree scoped by an ingest run:

```text
ingest_runs.run_id
  file_locations.root_path
    files.file_hash
      document_pages.page_no
        document_annotation_identities.annotation_id
          document_regions
          document_text_region_links
```

Preview images are file/page assets. OCR text, text spans, annotation boxes, and
region overlays are run-scoped outputs and must be read with `run_id`.

## Folder And File

`ingest_runs.root_path` is the selected folder for a scan. `file_locations`
records where a content hash was observed:

- `root_path`: selected scan root.
- `absolute_path`: full local path at discovery time.
- `relative_path`: path shown in the Workbench explorer.
- `file_hash`: content identity shared with `files`.

`files.file_hash` is the document identity. It stores document-level metadata
such as `display_name`, `extension`, `size_bytes`, `page_count`, `status`, and
timestamps. A file can appear in multiple runs and can have multiple observed
locations, but `file_hash` remains the stable file key.

## Page

`document_pages` is keyed by `file_hash` and numeric `page_no`. The page row is
for render and processing state: dimensions, DPI, status, and page errors.

`document_preview_images` is also keyed by `file_hash`, `page_no`, and
`variant`. These images are reusable file/page assets. They are not run-scoped,
so the Workbench can display the same page image while changing the selected
run's OCR overlays.

## Run

`ingest_runs.run_id` is a UUID v7 primary id. A new OCR scan creates a new run,
even if it scans the same files. `ingest_run_documents` records the file order
inside that run.

Run-scoped OCR outputs use `run_id` plus `file_hash` plus `page_no`:

- `document_run_page_ocr`: page text and OCR status for one run.
- `document_regions`: annotation boxes for one run.
- `document_text_region_links`: text span boundaries for one run.
- `ocr_stream_events`: replayable realtime events for one run.

Reads that render OCR output must pass `run_id`. Missing `run_id` support exists
only for compatibility and must not be used by new Workbench flows.

## Annotation

`document_annotation_identities.annotation_id` is the canonical persisted region
identity. It is assigned when a bounding box is discovered, before text content
is final. The id is UUID v7 and is independent from label, text, and geometry.

`source_region_key` is an internal deterministic lookup key for resolving the
same discovered region during batched writes or migration. It is not a UI id.

`document_regions.region_id` is a compatibility field. New OCR data sets it to
the same value as `annotation_id`, but application code should select,
cross-link, and render using `annotation_id`.

`document_text_region_links.annotation_id` binds a text scope to the same
annotation. A text scope starts at the region marker and ends at the next region
marker or the end of the page text. This lets the text preview highlight the
same content range that the image/PDF overlay represents.

## API Contract

Run-scoped reads return canonical annotation ids:

- `/api/documents/{file_hash}/regions?run_id={run_id}` returns `boxes[]` with
  `annotation_id`, `region_id`, geometry, label, and content.
- `/api/documents/{file_hash}/text?run_id={run_id}` returns `pages[].spans[]`
  with `annotation_id`, `region_id`, and text offsets.

The snippet route is still named
`/api/documents/{file_hash}/regions/{region_id}/snippet` for route
compatibility. New callers should pass the canonical `annotation_id`; current
new OCR rows have `region_id == annotation_id`.

## React Contract

React selection state uses the canonical annotation id as `regionId` until the
client types can be renamed without a broad API break. All new Workbench
cross-focus behavior should resolve `annotation_id` first and only fall back to
`region_id` for older fixtures or payloads.

The annotation UUID must appear once in each primary DOM element:

```text
id="annotation-box-{annotation_id}"
id="annotation-text-{annotation_id}"
```

Do not duplicate the UUID in `data-annotation-id`, `data-region-id`, title text,
or accessible labels. The element id is the only DOM selector contract. Helper
functions live in `src/trapo-client/src/api/annotationIdentity.ts`.

Manual focus must be deterministic:

- clicking an overlay box focuses `annotation-text-{annotation_id}`;
- clicking a text `#` anchor focuses `annotation-box-{annotation_id}`;
- manual focus is not debounced or throttled;
- focused text scopes may show a short-lived visual indicator from one `#`
  anchor until the next `#` anchor.

Realtime OCR auto-follow can be throttled to avoid jitter, but it must never
change the canonical identity mapping.
