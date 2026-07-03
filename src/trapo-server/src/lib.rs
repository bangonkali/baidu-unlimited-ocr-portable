pub mod app;
pub mod catalog;
pub mod config;
pub mod error;
pub mod folder_dialog;
pub mod logger;
pub mod ocr;
pub mod openapi;
pub mod pdf;
pub mod realtime;
pub mod routes;
pub mod scanner;
pub mod storage;
pub mod types;
pub mod workbench_diagnostics_types;
pub mod workbench_types;

pub use app::{AppState, build_router};
pub use config::ServerConfig;
