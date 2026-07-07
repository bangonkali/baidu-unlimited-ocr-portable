#[cfg(test)]
mod rag_chunk_tests {
    use super::*;

    #[test]
    fn cjk_text_is_chunked_under_embedding_batch_limit() {
        let text = "饮用水卫生标准".repeat(180);
        let chunks = rag_text_chunks(&text);

        assert!(chunks.len() > 1);
        for (start, end) in chunks {
            let chunk = &text[start..end];
            assert!(estimate_tokens(chunk) <= RAG_SEGMENT_TARGET_TOKENS);
        }
    }

    #[test]
    fn append_page_segments_preserves_offsets_for_chunks() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: "水".repeat(900),
                spans: Vec::new(),
            }],
            &mut segments,
        );

        assert!(segments.len() > 1);
        assert_eq!(segments[0].source_kind, "page_chunk");
        assert_eq!(segments[0].text_start, 0);
        assert!(segments[0].text_end > segments[0].text_start);
        assert!(segments
            .iter()
            .all(|segment| segment.token_estimate <= RAG_SEGMENT_TARGET_TOKENS));
    }

    #[test]
    fn stale_or_oversized_text_segments_are_rebuilt() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: "short text".to_string(),
                spans: Vec::new(),
            }],
            &mut segments,
        );

        assert!(rag_text_segments_are_current(&segments));
        segments[0].source_kind = "page".to_string();
        assert!(!rag_text_segments_are_current(&segments));
        segments[0].source_kind = "page_chunk".to_string();
        segments[0].token_estimate = RAG_SEGMENT_TARGET_TOKENS + 1;
        assert!(!rag_text_segments_are_current(&segments));
    }

    #[test]
    fn detector_marker_only_page_creates_no_rag_text_segments() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: "<|det|>image [164, 0, 842, 999]<|/det|>".to_string(),
                spans: Vec::new(),
            }],
            &mut segments,
        );

        assert!(segments.is_empty());
    }

    #[test]
    fn category_prefixes_are_stored_as_segment_categories() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: concat!(
                    "header Quarterly results\n",
                    "text Revenue grew across all product lines.\n",
                    "aside Preliminary unaudited statement\n",
                    "<|det|>image [164, 0, 842, 999]<|/det|>\n"
                )
                .to_string(),
                spans: Vec::new(),
            }],
            &mut segments,
        );

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].category, "header");
        assert_eq!(segments[0].text, "Quarterly results");
        assert_eq!(segments[0].text_start, "header ".len() as u64);
        assert_eq!(segments[1].category, "text");
        assert_eq!(segments[1].text, "Revenue grew across all product lines.");
        assert_eq!(segments[2].category, "aside");
        assert_eq!(segments[2].text, "Preliminary unaudited statement");
        assert!(segments
            .iter()
            .all(|segment| !segment.text.contains("<|det|>")));
        assert!(rag_text_segments_are_current(&segments));
    }

    #[test]
    fn appended_segments_preserve_overlapping_annotation_id() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: "title Annual report\ntext Revenue grew.".to_string(),
                spans: vec![
                    crate::workbench_types::TextRegionSpan {
                        annotation_id: "018f7a9b-10a0-7aa0-8f00-100000000001".to_string(),
                        end: 19,
                        page_no: 1,
                        region_id: "legacy-region-a".to_string(),
                        source_region_key: "source-a".to_string(),
                        start: 6,
                    },
                    crate::workbench_types::TextRegionSpan {
                        annotation_id: "018f7a9b-10a0-7aa0-8f00-100000000002".to_string(),
                        end: 38,
                        page_no: 1,
                        region_id: "legacy-region-b".to_string(),
                        source_region_key: "source-b".to_string(),
                        start: 25,
                    },
                ],
            }],
            &mut segments,
        );

        assert_eq!(
            segments[0].annotation_id.as_deref(),
            Some("018f7a9b-10a0-7aa0-8f00-100000000001")
        );
        assert_eq!(
            segments[1].annotation_id.as_deref(),
            Some("018f7a9b-10a0-7aa0-8f00-100000000002")
        );
    }

    #[test]
    fn stale_marker_or_category_prefixed_segments_are_rebuilt() {
        let mut segments = Vec::new();
        append_page_segments(
            "run-a",
            "file-a",
            vec![PageTextRecord {
                page_no: 1,
                text: "plain text".to_string(),
                spans: Vec::new(),
            }],
            &mut segments,
        );

        segments[0].text = "<|det|>image [1,2,3,4]<|/det|>".to_string();
        assert!(!rag_text_segments_are_current(&segments));
        segments[0].text = "image [1,2,3,4]".to_string();
        assert!(!rag_text_segments_are_current(&segments));
    }
}
