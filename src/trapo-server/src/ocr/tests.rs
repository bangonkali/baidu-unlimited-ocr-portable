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
        assert_eq!(parsed.cleaned_text, "A Title B");
        assert_eq!(parsed.boxes.len(), 1);
        assert_eq!(parsed.spans[0].start, 2);
        assert_eq!(parsed.spans[0].end, 2);
        assert_eq!(parsed.boxes[0].content_markdown, "Title B");
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

        assert_eq!(parsed.cleaned_text, "First body Second tail");
        assert_eq!(parsed.boxes[0].content_markdown, "First body");
        assert_eq!(parsed.boxes[1].content_markdown, "Second tail");
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
        assert_close(parsed.boxes[0].left_percent, 100.0 / 999.0 * 100.0);
        assert_close(parsed.boxes[0].width_percent, 800.0 / 999.0 * 100.0);
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
