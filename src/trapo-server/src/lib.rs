//! Library surface for the local Trapo OCR workbench server.

pub(crate) mod app;
pub(crate) mod catalog;
pub(crate) mod config;
pub(crate) mod embedding;
pub(crate) mod error;
pub(crate) mod folder_dialog;
pub(crate) mod ids;
pub(crate) mod logger;
pub(crate) mod ocr;
pub(crate) mod openapi;
pub(crate) mod pdf;
pub(crate) mod realtime;
pub(crate) mod realtime_event_types;
pub(crate) mod routes;
pub(crate) mod scanner;
pub(crate) mod shutdown;
pub(crate) mod storage;
pub(crate) mod types;
pub(crate) mod workbench_diagnostics_types;
pub(crate) mod workbench_types;

pub use app::{AppState, build_router};
pub use config::ServerConfig;
pub use error::{AppError, Result};
pub use ocr::validate_ffi_library;
pub use openapi::ApiDoc;
pub use storage::Repository;
