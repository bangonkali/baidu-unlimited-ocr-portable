# Current Skylos Triage

Raw input: `.logs/skylos-first-party-20260704-r3.raw.json`

| State | Count |
| --- | ---: |
| Open | 0 |
| Accepted exception | 297 |
| Excluded by scope | 55 |

## Open Findings

No open first-party findings remain.

## Accepted Findings

| Category | Rule | Path | Line | Name | Exception |
| --- | --- | --- | ---: | --- | --- |
| danger | SKY-D216 | `scripts/install_runtime.py` | 295 |  | runtime-installer-github-network |
| danger | SKY-D216 | `scripts/install_runtime.py` | 308 |  | runtime-installer-github-network |
| danger | SKY-D325 | `scripts/install_runtime.py` | 311 |  | runtime-installer-bounded-local-files |
| danger | SKY-D325 | `scripts/install_runtime.py` | 598 |  | runtime-installer-bounded-local-files |
| danger | SKY-D215 | `scripts/install_runtime.py` | 626 |  | runtime-installer-bounded-paths |
| danger | SKY-D215 | `scripts/install_runtime.py` | 631 |  | runtime-installer-bounded-paths |
| danger | SKY-D215 | `scripts/linux/setup-build.sh` | 512 |  | setup-build-repo-local-paths |
| danger | SKY-D215 | `scripts/linux/setup-build.sh` | 651 |  | setup-build-repo-local-paths |
| danger | SKY-D215 | `scripts/mac/setup-build.sh` | 709 |  | setup-build-repo-local-paths |
| danger | SKY-D215 | `scripts/mac/setup-build.sh` | 851 |  | setup-build-repo-local-paths |
| danger | SKY-D325 | `scripts/package_runtime.py` | 27 |  | package-runtime-local-files |
| danger | SKY-D325 | `scripts/package_runtime.py` | 414 |  | package-runtime-local-files |
| danger | SKY-D216 | `scripts/package_trapo_workbench.py` | 116 |  | workbench-packager-github-network |
| danger | SKY-D326 | `scripts/package_trapo_workbench.py` | 137 |  | workbench-packager-archive-extract |
| danger | SKY-D215 | `scripts/package_trapo_workbench.py` | 237 |  | workbench-packager-stage-paths |
| danger | SKY-D215 | `scripts/package_trapo_workbench.py` | 240 |  | workbench-packager-stage-paths |
| danger | SKY-D324 | `scripts/package_trapo_workbench.py` | 256 |  | workbench-packager-stage-writes |
| danger | SKY-D325 | `scripts/skylos_triage.py` | 28 |  | skylos-triage-local-artifacts |
| danger | SKY-D324 | `scripts/skylos_triage.py` | 160 |  | skylos-triage-local-reports |
| danger | SKY-D325 | `scripts/test_ctypes_runtime.py` | 78 |  | ctypes-runtime-local-abi-read |
| danger | SKY-D216 | `src/trapo-client/src/api/http.ts` | 16 |  | client-api-same-origin-fetch |
| danger | SKY-D216 | `src/trapo-client/src/api/http.ts` | 26 |  | client-api-same-origin-fetch |
| danger | SKY-D216 | `src/trapo-client/src/api/http.ts` | 42 |  | client-api-same-origin-fetch |
| danger | SKY-D253 | `src/trapo-client/src/features/workbench/useWorkbenchPageController.ts` | 278 |  | client-public-document-id-comparisons |
| danger | SKY-D253 | `src/trapo-client/src/features/workbench/useWorkbenchRouteSync.ts` | 49 |  | client-public-document-id-comparisons |
| danger | SKY-D253 | `src/trapo-client/src/features/workbench/useWorkbenchRouteSync.ts` | 48 |  | client-public-document-id-comparisons |
| danger | SKY-D253 | `src/trapo-client/src/features/workbench/useWorkbenchSelectionActions.ts` | 54 |  | client-public-document-id-comparisons |
| danger | SKY-D215 | `src/trapo-server/build.rs` | 85 |  | cargo-build-script-local-paths |
| danger | SKY-D215 | `src/trapo-server/build.rs` | 87 |  | cargo-build-script-local-paths |
| danger | SKY-D215 | `src/trapo-server/src/app/ingest_start.rs` | 56 |  | server-ingest-validated-local-documents |
| danger | SKY-D215 | `src/trapo-server/src/app/ingest_start.rs` | 73 |  | server-ingest-validated-local-documents |
| danger | SKY-D215 | `src/trapo-server/src/bin/export-openapi.rs` | 15 |  | server-openapi-explicit-output |
| danger | SKY-D212 | `src/trapo-server/src/catalog/runtime.rs` | 88 |  | server-fixed-runtime-probes |
| danger | SKY-D215 | `src/trapo-server/src/logger.rs` | 22 |  | server-log-root-path |
| danger | SKY-D212 | `src/trapo-server/src/main.rs` | 96 |  | server-fixed-browser-opener |
| danger | SKY-D215 | `src/trapo-server/src/ocr/ffi_engine.rs` | 85 |  | server-ffi-local-runtime-paths |
| danger | SKY-D215 | `src/trapo-server/src/ocr/ffi_loader.rs` | 135 |  | server-ffi-local-runtime-paths |
| danger | SKY-D215 | `src/trapo-server/src/pdf.rs` | 62 |  | server-pdf-validated-local-ingest |
| danger | SKY-D215 | `src/trapo-server/src/pdf.rs` | 73 |  | server-pdf-validated-local-ingest |
| quality | SKY-L027 | `scripts/install_runtime.py` | 639 | utf-8 | python-release-script-complexity |
| quality | SKY-L027 | `scripts/install_runtime.py` | 702 | store_true | python-release-script-complexity |
| quality | SKY-L028 | `scripts/install_runtime.py` | 151 | probe_accelerator | python-release-script-complexity |
| quality | SKY-Q306 | `scripts/install_runtime.py` | 189 | detect_platform | python-release-script-complexity |
| quality | SKY-L028 | `scripts/install_runtime.py` | 189 | detect_platform | python-release-script-complexity |
| quality | SKY-P403 | `scripts/install_runtime.py` | 376 | nested_loop | python-release-script-complexity |
| quality | SKY-Q306 | `scripts/install_runtime.py` | 501 | installed_paths | python-release-script-complexity |
| quality | SKY-Q301 | `scripts/install_runtime.py` | 561 | install_runtime | python-release-script-complexity |
| quality | SKY-Q306 | `scripts/install_runtime.py` | 561 | install_runtime | python-release-script-complexity |
| quality | SKY-Q302 | `scripts/install_runtime.py` | 561 | install_runtime | python-release-script-complexity |
| quality | SKY-C304 | `scripts/install_runtime.py` | 561 | install_runtime | python-release-script-complexity |
| quality | SKY-L007 | `scripts/install_runtime.py` | 610 | except | python-release-script-complexity |
| quality | SKY-L030 | `scripts/install_runtime.py` | 610 | except | python-release-script-complexity |
| quality | SKY-L027 | `scripts/package_runtime.py` | 87 | Release | python-release-script-complexity |
| quality | SKY-L027 | `scripts/package_runtime.py` | 343 | utf-8 | python-release-script-complexity |
| quality | SKY-P401 | `scripts/package_runtime.py` | 40 | read | python-release-script-complexity |
| quality | SKY-Q306 | `scripts/package_runtime.py` | 116 | collect_runtime_files | python-release-script-complexity |
| quality | SKY-P403 | `scripts/package_runtime.py` | 134 | nested_loop | python-release-script-complexity |
| quality | SKY-Q306 | `scripts/package_runtime.py` | 254 | package_runtime | python-release-script-complexity |
| quality | SKY-C304 | `scripts/package_runtime.py` | 254 | package_runtime | python-release-script-complexity |
| quality | SKY-T102 | `scripts/skylos_triage.py` | 87 | iter_findings | python-release-script-complexity |
| quality | SKY-P403 | `scripts/skylos_triage.py` | 89 | nested_loop | python-release-script-complexity |
| quality | SKY-P403 | `scripts/skylos_triage.py` | 171 | nested_loop | python-release-script-complexity |
| quality | SKY-P403 | `scripts/test_ctypes_runtime.py` | 134 | nested_loop | python-release-script-complexity |
| quality | SKY-L007 | `scripts/test_ctypes_runtime.py` | 131 | except | python-release-script-complexity |
| quality | SKY-L030 | `scripts/test_ctypes_runtime.py` | 131 | except | python-release-script-complexity |
| quality | SKY-Q306 | `scripts/test_ctypes_runtime.py` | 195 | load_ffi_for_abi | python-release-script-complexity |
| quality | SKY-C304 | `src/trapo-client/src/components/workbench/Tree/Tree.tsx` | 60 | TreeViewRow | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/components/workbench/Tree/Tree.tsx` | 120 | TreeGridRow | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/commands/appCommands.ts` | 168 | modelCommands | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/commands/appCommands.ts` | 169 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/commands/CommandPalette.tsx` | 35 | CommandPalette | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/DiagnosticsPanel.tsx` | 39 | DiagnosticsPanel | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/IngestStartPanel.test.tsx` | 7 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/IngestStartPanel.tsx` | 39 | IngestStartPanel | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/ModelActions.tsx` | 18 | ModelActions | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/ModelDataGrid.tsx` | 24 | ModelDataGrid | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/ModelDetailPanel.tsx` | 17 | ModelDetailPanel | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/ModelManager.test.tsx` | 20 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/ModelManager.tsx` | 124 | ModelToolbar | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/ModelManager.tsx` | 39 | ModelManager | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/NotificationBell.tsx` | 7 | NotificationBell | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/PreviewPane.tsx` | 40 | PreviewPane | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/PreviewPane.tsx` | 94 | PagePreview | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/SettingsPanel.tsx` | 22 | SettingsPanel | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/SettingsPanel.tsx` | 147 | OcrSettings | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/TextPane.tsx` | 20 | TextPane | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/TraceableMarkdown.test.tsx` | 7 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/useWorkbenchCommands.ts` | 32 | useWorkbenchCommands | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/useWorkbenchIngestActions.test.ts` | 13 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/useWorkbenchIngestActions.test.ts` | 14 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/useWorkbenchIngestActions.ts` | 13 | useWorkbenchIngestActions | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/useWorkbenchPageController.ts` | 74 | useWorkbenchPageController | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/useWorkbenchPageController.ts` | 196 | useWorkbenchActions | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/WorkbenchPageSupport.tsx` | 90 | usePersistedWorkbenchUiSettings | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/WorkbenchPanels.tsx` | 42 | WorkbenchPanels | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/features/workbench/WorkbenchViewContent.tsx` | 77 | WorkbenchViewContent | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/realtime/ocrReplayProjection.ts` | 36 | projectOcrReplayEvents | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/realtime/RealtimeBridge.tsx` | 33 | createRealtimeDispatcher | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/realtime/realtimeEvents.test.ts` | 78 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/realtime/realtimeEvents.test.ts` | 20 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/realtime/realtimeEvents.test.ts` | 136 | anonymous | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/realtime/realtimeEvents.test.ts` | 177 | anonymous | react-composition-length |
| quality | SKY-Q301 | `src/trapo-client/src/realtime/realtimeQueryBridge.ts` | 93 | applyRealtimeEventToQueryClient | react-state-branching |
| quality | SKY-C304 | `src/trapo-client/src/realtime/realtimeQueryBridge.ts` | 93 | applyRealtimeEventToQueryClient | react-composition-length |
| quality | SKY-C304 | `src/trapo-client/src/routeSearch.test.ts` | 11 | anonymous | react-composition-length |
| quality | SKY-L004 | `tests/test_runtime_installer.py` | 24 | try | python-test-shape |
| quality | SKY-R104 | `.` | 1 | pre-commit-policy | root-policy-precommit |
| unused_files | SKY-E003 | `src/trapo-client/.storybook/preview.ts` | 1 |  | storybook-preview-entrypoint |
| unused_files | SKY-E003 | `src/trapo-client/src/components/ui/command.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/commands/CommandPalette.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/ActivityBar.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/GuidedTour.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/NotificationBell.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/StartHere.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/StatusBar.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/TextPane.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/WorkbenchPage.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/WorkbenchPageSupport.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/WorkbenchPanels.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/WorkbenchViewContent.tsx` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/startOcrEntry.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/useModelRouteActions.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/useOcrReplayHydration.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/useWorkbenchCommands.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/useWorkbenchPageController.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/useWorkbenchSelectionActions.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/workbenchContentProps.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/features/workbench/workbenchRouteState.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/__root.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/diagnostics.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/ingest.start.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/models.$modelId.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/models.downloads.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/models.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/settings.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/routes/workbench.tsx` | 1 |  | tanstack-router-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/stores/notificationStore.ts` | 1 |  | react-framework-dead-code |
| unused_files | SKY-E003 | `src/trapo-client/src/stories/ComponentInventory.stories.tsx` | 1 |  | storybook-entrypoints |
| unused_files | SKY-E003 | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 1 |  | storybook-entrypoints |
| unused_functions |  | `src/trapo-client/src/features/workbench/ModelGridCells.tsx` | 39 | FilesCell | react-public-cells |
| unused_functions |  | `src/trapo-client/src/features/workbench/ModelGridCells.tsx` | 21 | ProgressCell | react-public-cells |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 159 | health_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 162 | status_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 165 | openapi_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 168 | folder_dialog_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 171 | start_ingest_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 174 | list_runs_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 177 | get_run_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 180 | stop_run_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 183 | run_events_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 186 | ocr_events_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 189 | diagnostics_runs_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 192 | diagnostics_trace_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 195 | diagnostics_progress_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 198 | diagnostics_analytics_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 201 | diagnostics_models_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 204 | run_metrics_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 207 | recent_metrics_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 210 | list_documents_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 213 | search_documents_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 216 | get_document_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 219 | document_regions_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 222 | region_snippet_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 225 | document_text_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 228 | preview_images_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 231 | preview_image_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 234 | settings_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 237 | update_settings_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 240 | models_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 243 | download_model_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 246 | select_model_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 249 | cancel_model_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 252 | model_events_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/openapi.rs` | 255 | logs_doc | utoipa-openapi-builders |
| unused_functions |  | `src/trapo-server/src/routes_diagnostics.rs` | 43 | diagnostics_runs | rust-route-macro-handlers |
| unused_functions |  | `src/trapo-server/src/routes_models.rs` | 16 | select_model | rust-route-macro-handlers |
| unused_imports |  | `src/trapo-server/src/app/region_snippets.rs` | 1 | GenericImageView | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 1 | Path | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 1 | UNIX_EPOCH | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 8 | Utc | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 9 | Value | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DEFAULT_MODEL_ID | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DEFAULT_PROFILE_ID | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | PROVIDER_LABEL | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | PROVIDER_REPO_ID | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | PROVIDER_REVISION | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | RETRY_PROFILE_ID | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | SHARED_MMPROJ_FILE | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | SHARED_MMPROJ_SIZE_BYTES | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | choose_runtime_id | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | find_model | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | find_profile | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | model_catalog | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ocr_profiles | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | runtime_record | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | runtime_variants | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | AppError | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | Result | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | is_pdf | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiscoveredFile | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | SUPPORTED_INPUTS | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | discover_supported_files | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | generic_path | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | stable_hash | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticEventInsert | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticEventRow | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticModelLeaseRow | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticRunRow | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticSpanInsert | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticSpanRow | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticTraceFilter | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticWorkUnitRow | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | OcrPageMetrics | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | StoredDocument | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | StoredPage | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | StoredRealtimeEvent | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | StoredRun | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | WorkUnitUpsert | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | HealthPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ModelAssetRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ModelDownloadEvent | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ModelDownloadFileRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ModelDownloadRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ModelDownloadRequest | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ModelSelectRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | ModelsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | SettingsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | SettingsUpdateRequest | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | StatusPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticAnalyticsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticAnalyticsSummary | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticBreakdownRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticEventRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticModelLeaseRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticModelsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticProgressPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticProgressSummary | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticRecommendationRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticRunRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticRunsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticSlowSpanRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticSpanRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticTracePayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticTraceSummary | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DiagnosticWorkUnitRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DocumentDetail | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DocumentRegionsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DocumentSummary | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DocumentTextPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | DocumentsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | FolderDialogResponse | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | IngestRunRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | IngestRunsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | IngestStartRequest | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | IngestStartResponse | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | LogsPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | OcrMetricsTreeNode | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | OcrMetricsTreePayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | OcrReplayPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | PageTextRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | PreviewImagesPayload | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/app.rs` | 12 | RealtimeEventRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/bin/export-openapi.rs` | 4 | OpenApi | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/catalog.rs` | 1 | Path | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/catalog.rs` | 3 | OcrProfileRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/catalog.rs` | 3 | RuntimeVariantRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/folder_dialog.rs` | 282 | ExitStatusExt | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/logger.rs` | 1 | BufRead | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 1 | CStr | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 1 | CString | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 1 | size_of | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 1 | Path | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 1 | LazyLock | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 9 | Regex | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 11 | RuntimeVariant | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 11 | AppError | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 11 | Result | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 11 | region_hash_key | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/ocr.rs` | 11 | OcrProfileRecord | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/pdf.rs` | 3 | GenericImageView | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/routes.rs` | 1 | IntoResponse | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/routes.rs` | 11 | Servable | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/scanner.rs` | 1 | _ | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/storage.rs` | 8 | Utc | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/storage.rs` | 9 | OptionalExt | rust-module-reexports |
| unused_imports |  | `src/trapo-server/src/storage.rs` | 12 | Result | rust-module-reexports |
| unused_variables |  | `src/trapo-client/src/routes/__root.tsx` | 3 | Route | tanstack-router-route-exports |
| unused_variables |  | `src/trapo-client/src/stories/ComponentInventory.stories.tsx` | 29 | Inventory | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 143 | Models | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 156 | ModelsDownloading | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 169 | ModelDownloads | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 233 | fixturePreviewImage | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 64 | Explorer | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 76 | Traceability | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 38 | Ingest | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 104 | LiveText | storybook-named-exports |
| unused_variables |  | `src/trapo-client/src/stories/WorkbenchSurfaces.stories.tsx` | 184 | ModelDetail | storybook-named-exports |
