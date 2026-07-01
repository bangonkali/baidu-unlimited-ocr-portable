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
        assert_eq!(parsed.boxes[0].content_markdown, "Title");
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
        assert_eq!(parsed.boxes[0].left_percent, 100.0 / 999.0 * 100.0);
        assert_eq!(parsed.boxes[0].width_percent, 800.0 / 999.0 * 100.0);
    }
}
