fn append_page_segments(
    source_run_id: &str,
    file_hash: &str,
    pages: Vec<PageTextRecord>,
    segments: &mut Vec<RagTextSegmentRow>,
) {
    for page in pages {
        for source in rag_text_sources(&page.text) {
            for (start, end) in rag_text_chunks(&source.text) {
                let slice = &source.text[start..end];
                let chunk = slice.trim();
                if chunk.is_empty() {
                    continue;
                }
                let leading_trim = slice.len() - slice.trim_start().len();
                let trailing_trim = slice.trim_end().len();
                let text_start = source.offset.saturating_add(start).saturating_add(leading_trim);
                let text_end = source.offset.saturating_add(start).saturating_add(trailing_trim);
                let segment_index = usize_to_u32_saturating(segments.len());
                segments.push(RagTextSegmentRow { // skylos: ignore[SKY-D215] file_hash is a digest key, not a filesystem path.
                    segment_id: new_persistence_id(),
                    source_run_id: source_run_id.to_string(),
                    file_hash: file_hash.to_string(),
                    page_no: page.page_no,
                    segment_index,
                    annotation_id: None,
                    category: source.category.clone(),
                    text: chunk.to_string(),
                    token_estimate: estimate_tokens(chunk),
                    text_start: usize_to_u64_saturating(text_start),
                    text_end: usize_to_u64_saturating(text_end),
                    source_kind: "page_chunk".to_string(),
                });
            }
        }
    }
}

const RAG_SEGMENT_TARGET_TOKENS: u32 = 192;
const RAG_SEGMENT_OVERLAP_TOKENS: u32 = 32;
const TOKEN_UNITS_PER_TOKEN: u32 = 4;
const RAG_TEXT_CATEGORIES: &[&str] = &[
    "aside",
    "caption",
    "figure",
    "footer",
    "header",
    "image",
    "paragraph",
    "section",
    "table",
    "text",
    "title",
];

#[derive(Debug, Clone)]
struct RagTextSource {
    category: String,
    offset: usize,
    text: String,
}

struct CategorySplit<'a> {
    category: String,
    clean_offset: usize,
    clean_text: &'a str,
}

fn rag_text_segments_are_current(segments: &[RagTextSegmentRow]) -> bool {
    !segments.is_empty()
        && segments.iter().all(|segment| {
            segment.source_kind == "page_chunk"
                && segment.token_estimate <= RAG_SEGMENT_TARGET_TOKENS
                && rag_segment_text_is_clean(&segment.text)
        })
}

fn rag_text_sources(raw_text: &str) -> Vec<RagTextSource> {
    let text = remove_marker_blocks(raw_text);
    let mut sources = Vec::new();
    let mut cursor = 0;
    for line in text.split_inclusive('\n') {
        let line_start = cursor;
        cursor += line.len();
        let trimmed_start = line.len() - line.trim_start().len();
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let category_split = split_category_prefix(trimmed);
        let clean_text = category_split.clean_text;
        if clean_text.is_empty() {
            continue;
        }
        sources.push(RagTextSource {
            category: category_split.category,
            offset: line_start
                .saturating_add(trimmed_start)
                .saturating_add(category_split.clean_offset),
            text: clean_text.to_string(),
        });
    }
    sources
}

fn rag_segment_text_is_clean(text: &str) -> bool {
    !text.contains("<|det|>")
        && !text.contains("<|/det|>")
        && !text.contains("<|ref|>")
        && !text.contains("<|/ref|>")
        && !has_category_prefix(text)
}

fn split_category_prefix(text: &str) -> CategorySplit<'_> {
    for category in RAG_TEXT_CATEGORIES {
        if text.eq_ignore_ascii_case(category) {
            return CategorySplit {
                category: (*category).to_string(),
                clean_offset: text.len(),
                clean_text: "",
            };
        }
        if let Some((clean_offset, rest)) = strip_category_prefix(text, category) {
            return CategorySplit {
                category: (*category).to_string(),
                clean_offset,
                clean_text: rest,
            };
        }
    }
    CategorySplit {
        category: "page_text".to_string(),
        clean_offset: 0,
        clean_text: text,
    }
}

fn has_category_prefix(text: &str) -> bool {
    RAG_TEXT_CATEGORIES
        .iter()
        .any(|category| strip_category_prefix(text.trim(), category).is_some())
}

fn strip_category_prefix<'a>(text: &'a str, category: &str) -> Option<(usize, &'a str)> {
    let prefix = text.get(..category.len())?;
    if text.len() <= category.len() || !prefix.eq_ignore_ascii_case(category) {
        return None;
    }
    let rest = &text[category.len()..];
    let separator_len = rest
        .chars()
        .next()
        .filter(|character| {
            character.is_whitespace() || matches!(character, ':' | '-' | '|' | '.')
        })
        .map(char::len_utf8)?;
    let clean = rest[separator_len..].trim_start_matches(|character: char| {
        character.is_whitespace() || matches!(character, ':' | '-' | '|')
    });
    Some((text.len() - clean.len(), clean))
}

fn remove_marker_blocks(text: &str) -> String {
    let without_refs = remove_tagged_blocks(text, "<|ref|>", "<|/ref|>");
    remove_tagged_blocks(&without_refs, "<|det|>", "<|/det|>")
}

fn remove_tagged_blocks(text: &str, open: &str, close: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    while let Some(relative_start) = text[cursor..].find(open) {
        let start = cursor + relative_start;
        output.push_str(&text[cursor..start]);
        let content_start = start + open.len();
        let Some(relative_end) = text[content_start..].find(close) else {
            cursor = text.len();
            break;
        };
        cursor = content_start + relative_end + close.len();
    }
    output.push_str(&text[cursor..]);
    output
}

fn rag_text_chunks(text: &str) -> Vec<(usize, usize)> {
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let end = chunk_end(text, start);
        if end <= start {
            break;
        }
        chunks.push((start, end));
        if end == text.len() {
            break;
        }
        let overlapped = overlap_start(text, start, end);
        start = if overlapped <= start { end } else { overlapped };
    }
    chunks
}

fn chunk_end(text: &str, start: usize) -> usize {
    let target_units = RAG_SEGMENT_TARGET_TOKENS.saturating_mul(TOKEN_UNITS_PER_TOKEN);
    let minimum_boundary_units = target_units / 2;
    let mut units = 0_u32;
    let mut last_boundary = None;
    for (offset, character) in text[start..].char_indices() {
        let next = start + offset + character.len_utf8();
        units = units.saturating_add(char_token_units(character));
        if units >= minimum_boundary_units && is_chunk_boundary(character) {
            last_boundary = Some(next);
        }
        if units >= target_units {
            return last_boundary.filter(|boundary| *boundary > start).unwrap_or(next);
        }
    }
    text.len()
}

fn overlap_start(text: &str, start: usize, end: usize) -> usize {
    let target_units = RAG_SEGMENT_OVERLAP_TOKENS.saturating_mul(TOKEN_UNITS_PER_TOKEN);
    let mut units = 0_u32;
    let mut candidate = end;
    for (offset, character) in text[..end].char_indices().rev() {
        units = units.saturating_add(char_token_units(character));
        candidate = offset;
        if units >= target_units {
            break;
        }
    }
    if candidate <= start { end } else { candidate }
}

fn estimate_tokens(text: &str) -> u32 {
    let units = text
        .chars()
        .map(char_token_units)
        .fold(0_u32, u32::saturating_add);
    units
        .saturating_add(TOKEN_UNITS_PER_TOKEN - 1)
        .saturating_div(TOKEN_UNITS_PER_TOKEN)
}

fn char_token_units(character: char) -> u32 {
    if character.is_whitespace() {
        0
    } else if is_cjk(character) {
        TOKEN_UNITS_PER_TOKEN
    } else {
        1
    }
}

fn is_cjk(character: char) -> bool {
    matches!(
        u32::from(character),
        0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0x3040..=0x30FF
            | 0xAC00..=0xD7AF
    )
}

const fn is_chunk_boundary(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | ')'
                | ']'
                | '}'
                | '\u{3002}'
                | '\u{ff0c}'
                | '\u{ff1b}'
                | '\u{ff1a}'
                | '\u{ff01}'
                | '\u{ff1f}'
        )
}

