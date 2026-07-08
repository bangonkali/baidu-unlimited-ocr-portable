#![allow(
    dead_code,
    reason = "shared OCR output model is being introduced before every engine emits it directly"
)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ParsedOcrPage;
use crate::workbench_types::{OcrGeometry, OverlayBox, TextRegionSpan};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OcrDocumentOutput {
    pub(crate) pages: Vec<OcrPageOutput>,
    pub(crate) provenance: OcrEngineProvenance,
    pub(crate) warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OcrPageOutput {
    pub(crate) page_no: u32,
    pub(crate) text: String,
    pub(crate) annotations: Vec<OcrAnnotation>,
    pub(crate) spans: Vec<OcrTextSpan>,
    pub(crate) timing_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OcrAnnotation {
    pub(crate) annotation_id: Option<String>,
    pub(crate) source_region_key: String,
    pub(crate) label: String,
    pub(crate) category: String,
    pub(crate) geometry: OcrGeometry,
    pub(crate) text_span: Option<OcrTextSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OcrTextSpan {
    pub(crate) annotation_id: Option<String>,
    pub(crate) source_region_key: String,
    pub(crate) page_no: u32,
    pub(crate) start: u64,
    pub(crate) end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OcrEngineProvenance {
    pub(crate) engine_id: String,
    pub(crate) pipeline: String,
    pub(crate) model_id: Option<String>,
    pub(crate) runtime_id: Option<String>,
    pub(crate) metadata: Value,
}

impl OcrDocumentOutput {
    pub(crate) fn from_parsed_page(
        parsed: &ParsedOcrPage,
        provenance: OcrEngineProvenance,
    ) -> Self {
        Self {
            pages: vec![OcrPageOutput {
                page_no: parsed.boxes.first().map_or(1, |item| item.page_no),
                text: parsed.cleaned_text.clone(),
                annotations: parsed
                    .boxes
                    .iter()
                    .map(|item| annotation_from_overlay(item, &parsed.spans))
                    .collect(),
                spans: parsed.spans.iter().map(text_span_from_overlay).collect(),
                timing_ms: None,
            }],
            provenance,
            warnings: Vec::new(),
        }
    }
}

fn annotation_from_overlay(item: &OverlayBox, spans: &[TextRegionSpan]) -> OcrAnnotation {
    let text_span = spans
        .iter()
        .find(|span| span.source_region_key == item.source_region_key)
        .map(text_span_from_overlay);
    OcrAnnotation {
        annotation_id: Some(item.annotation_id.clone()),
        source_region_key: item.source_region_key.clone(),
        label: item.label.clone(),
        category: item.category.clone(),
        geometry: item.resolved_geometry(),
        text_span,
    }
}

fn text_span_from_overlay(span: &TextRegionSpan) -> OcrTextSpan {
    OcrTextSpan {
        annotation_id: Some(span.annotation_id.clone()),
        source_region_key: span.source_region_key.clone(),
        page_no: span.page_no,
        start: span.start,
        end: span.end,
    }
}
