#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ref_det_markers() {
        let mut parsed = parse_ocr_markers(
            "A <|ref|>Title<|/ref|><|det|>[[0,0,999,100]]<|/det|> B",
            &ParseContext {
                file_hash: "hash".to_string(),
                page_no: 1,
                engine_id: "engine".to_string(),
                profile_id: "profile".to_string(),
            },
        );
        apply_region_content(&mut parsed);
        assert_eq!(parsed.cleaned_text, "A B");
        assert_eq!(parsed.boxes.len(), 1);
        assert_eq!(parsed.spans[0].start, 2);
        assert_eq!(parsed.spans[0].end, 2);
        assert_eq!(parsed.boxes[0].category, "Title");
        assert_eq!(parsed.boxes[0].content_markdown, "B");
    }

    #[test]
    fn region_content_uses_text_until_next_marker() {
        let mut parsed = parse_ocr_markers(
            "<|ref|>First<|/ref|><|det|>[[0,0,100,100]]<|/det|> body \
             <|ref|>Second<|/ref|><|det|>[[100,0,200,100]]<|/det|> tail",
            &ParseContext {
                file_hash: "hash".to_string(),
                page_no: 1,
                engine_id: "engine".to_string(),
                profile_id: "profile".to_string(),
            },
        );
        apply_region_content(&mut parsed);

        assert_eq!(parsed.cleaned_text, " body tail");
        assert_eq!(parsed.boxes[0].category, "First");
        assert_eq!(parsed.boxes[1].category, "Second");
        assert_eq!(parsed.boxes[0].content_markdown, "body");
        assert_eq!(parsed.boxes[1].content_markdown, "tail");
    }

    #[test]
    fn parses_standalone_det_marker() {
        let parsed = parse_ocr_markers(
            "<|det|>Cell [900,100,100,200]<|/det|>",
            &ParseContext {
                file_hash: "hash".to_string(),
                page_no: 1,
                engine_id: "engine".to_string(),
                profile_id: "profile".to_string(),
            },
        );
        assert_eq!(parsed.cleaned_text, "");
        assert_close(parsed.boxes[0].left_percent, 100.0 / 999.0 * 100.0);
        assert_close(parsed.boxes[0].width_percent, 800.0 / 999.0 * 100.0);
    }

    #[test]
    fn parses_geometry_sidecar_for_rotated_markers() -> Result<()> {
        let parsed = parse_ocr_markers(
            r#"<|ref|>Total<|/ref|><|det|>[[100,200,400,320]]<|/det|><|geom|>{"kind":"rotated_quad","points":[{"x":11.0,"y":20.0},{"x":41.0,"y":24.0},{"x":39.0,"y":34.0},{"x":9.0,"y":30.0}],"rotation_degrees":8.5,"layer_id":"source","coordinate_space":"page_percent","bounds":{"left":9.0,"top":20.0,"width":32.0,"height":14.0}}<|/geom|> total due"#,
            &ParseContext {
                file_hash: "hash".to_string(),
                page_no: 1,
                engine_id: "engine".to_string(),
                profile_id: "profile".to_string(),
            },
        );

        let Some(geometry) = parsed.boxes[0].geometry.as_ref() else {
            return Err(crate::error::AppError::Internal(
                "geometry sidecar should parse".to_string(),
            ));
        };
        assert_eq!(geometry.kind, "rotated_quad");
        assert_eq!(geometry.points.len(), 4);
        assert_eq!(geometry.rotation_degrees, Some(8.5));
        Ok(())
    }

    #[test]
    fn normalizes_parsed_markers_to_shared_output() {
        let mut parsed = parse_ocr_markers(
            "<|ref|>Total<|/ref|><|det|>[[10,20,110,60]]<|/det|> total due",
            &ParseContext {
                file_hash: "hash".to_string(),
                page_no: 2,
                engine_id: "unlimited-ocr".to_string(),
                profile_id: "profile".to_string(),
            },
        );
        apply_region_content(&mut parsed);
        let output = OcrDocumentOutput::from_parsed_page(
            &parsed,
            OcrEngineProvenance {
                engine_id: "unlimited-ocr".to_string(),
                pipeline: "marker".to_string(),
                model_id: Some("model".to_string()),
                runtime_id: Some("runtime".to_string()),
                metadata: serde_json::json!({}),
            },
        );

        assert_eq!(output.pages[0].page_no, 2);
        assert_eq!(output.pages[0].annotations[0].category, "Total");
        assert_eq!(output.pages[0].annotations[0].geometry.kind, "axis_aligned");
        assert_eq!(output.provenance.engine_id, "unlimited-ocr");
    }

    #[test]
    fn macos_preload_rank_orders_runtime_dependencies() {
        let mut names = [
            "libllama-common.0.dylib",
            "libmtmd.0.dylib",
            "libggml-metal.0.dylib",
            "libggml-base.0.dylib",
            "libllama.0.dylib",
            "libggml.0.dylib",
        ];

        names.sort_by_key(|name| (macos_dylib_preload_rank(name), *name));

        assert_eq!(names[0], "libggml-base.0.dylib");
        assert_eq!(names[1], "libggml.0.dylib");
        assert_eq!(names[5], "libllama-common.0.dylib");
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9);
    }
}
