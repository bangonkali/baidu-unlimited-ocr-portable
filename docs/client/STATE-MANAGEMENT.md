# Trapo Client State Management

## Scope

This document defines the client state architecture for `src/trapo-client`. It is based on the local TanStack Store documentation under `C:\Users\Bangonkali\Desktop\Projects\tanstack-store\docs` for the 0.14 API family: `createStore`/`Store`, selector subscriptions, framework selectors such as `TanStackStoreSelector`, and `batch`.

The goal is one coherent browser projection of backend state without duplicate long-lived copies of documents, runs, logs, model status, OCR text, or OCR regions.

## Ownership Model

TanStack Query owns server data. The Query cache is the canonical browser-side projection for API resources:

- `queryKeys.status`
- `queryKeys.runs`
- `queryKeys.documents(q)`
- `queryKeys.documentText(fileHash)`
- `queryKeys.documentRegions(fileHash)`
- `queryKeys.documentPreviewImages(fileHash)`
- `queryKeys.models`
- `queryKeys.logs`

TanStack Store owns local UI and realtime control state only:

- Workbench selection, panes, theme, visibility, guided tour, and local folder selection.
- Realtime connection state, last event timestamp, and visible sequence metadata.
- Notification UI state.

Do not mirror Query-owned data into TanStack Store. If a component needs documents, runs, regions, text, or logs, it reads the Query projection through hooks/selectors. Derived UI views are computed from Query data at render boundaries or memoized locally when needed.

## Realtime Pipeline

The default steady-state pipeline is:

1. Initial page load fetches Query resources once through normal hooks.
2. One global `/api/events` websocket mounts through `RealtimeBridge`.
3. `RealtimeBridge` serializes events through a single async dispatcher queue.
4. The dispatcher tracks `lastSeenSequence`, `lastAppliedSequence`, `lastAppliedOcrSequence`, and `lastRecoveredSequence`.
5. Typed event handlers update exact Query caches with `setQueryData`/`setQueriesData`.
6. Workbench Store setters update only UI state and return the existing object for no-op updates.

No steady-state polling should be used for status, runs, documents, logs, OCR replay, text, or regions. `/api/ocr/events` is reserved for initial selected-page hydration and recovery after reconnect or a sequence gap.

## Replay And Recovery

Replay is recovery, not synchronization.

- Initial websocket ready marks the current backend sequence as covered by initial HTTP hydration.
- Reconnect invalidates state-backed queries once, then replays missing OCR stream events when possible.
- OCR-relevant sequence gaps page `/api/ocr/events` with `limit=10000` and `next_since_sequence`.
- Recovered OCR events update caches through the same typed projection path as live events.
- Broad invalidation is only allowed for reconnect, unrecoverable desync, or manual refresh.

If replay cannot catch up because missing sequence numbers are non-OCR events or the replay window has moved, the dispatcher marks the gap recovered and continues applying live events. It does not invalidate state-backed queries for sparse replay. A replay fetch failure is treated as an unrecoverable recovery error and may invalidate state-backed queries once.

## Event Contract

The default browser stream should favor semantic UI events:

- `status.changed`
- `run.changed`
- `document.changed`
- `document.page.changed`
- `document.text.changed`
- `document.regions.changed`
- `ocr.page.text.patch`
- `ocr.page.region.upsert`
- `ocr.page.region.remove`
- `ocr.page.span.upsert`
- `ocr.page.span.remove`
- terminal OCR page events
- model and log events

`ocr.page.raw.delta` is not part of the default browser update stream. It can remain a diagnostic or future opt-in debug event, but normal workbench rendering must rely on chunked text patches plus final persisted page text/region data.

## Store Rules

TanStack Store updates must be referentially stable:

- Return the existing state object when a setter receives values already present in state.
- Use focused selectors in React components; avoid subscribing a large subtree to unrelated state.
- Keep server data out of Store to prevent duplicate memory ownership and inconsistent projections.
- Batch related local updates when multiple Store writes are needed from one user action.
- Use `shallow` or explicit field equality for object-shaped selectors and no-op guards.

## Query Projection Rules

Realtime handlers must update precise Query caches:

- Status events replace `queryKeys.status`.
- Run events upsert `queryKeys.runs` and update active status metadata.
- Document events upsert the unfiltered document list and update filtered lists only when the document is already present.
- Text and region events replace or patch the corresponding file-hash cache.
- Logs append to active log queries with a bounded list.
- Model events upsert model cache and selected model metadata.

Avoid `invalidateQueries` in per-event handlers. Invalidation is a recovery tool, not the normal data path.

## Testing Requirements

Client tests should cover:

- Dispatcher sequence-gap recovery calls `/api/ocr/events` once per actual gap.
- Replayed OCR events update Query caches without broad invalidation.
- Selected-page replay does not set a polling interval.
- Workbench Store setters keep the same state object for no-op updates.

Backend tests should cover:

- Default stream emits chunked `ocr.page.text.patch` events, not raw token deltas.
- Region/span events remain available when OCR markers complete.
- Final page persistence stores raw and cleaned text so no OCR content is lost.
