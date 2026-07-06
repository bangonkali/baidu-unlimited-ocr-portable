use crate::error::{AppError, Result};

use super::normalize_l2;

pub(super) fn too_many_tokens_error(model_id: &str, tokens: usize, max_tokens: usize) -> AppError {
    AppError::BadRequest(format!(
        "embedding text for {model_id} produced {tokens} tokens, above the safe llama.cpp encoder batch limit of {max_tokens}"
    ))
}

pub(super) fn split_text_near_middle(text: &str) -> Option<usize> {
    let midpoint = text.len() / 2;
    let mut best_boundary = None;
    let mut best_distance = usize::MAX;
    let mut first_after_midpoint = None;
    let mut last_before_midpoint = None;
    for (index, character) in text.char_indices() {
        let next = index + character.len_utf8();
        if next >= text.len() {
            break;
        }
        if next <= midpoint {
            last_before_midpoint = Some(next);
        } else if first_after_midpoint.is_none() {
            first_after_midpoint = Some(next);
        }
        if is_embedding_split_boundary(character) {
            let distance = next.abs_diff(midpoint);
            if distance < best_distance {
                best_distance = distance;
                best_boundary = Some(next);
            }
        }
    }
    best_boundary
        .or(first_after_midpoint)
        .or(last_before_midpoint)
        .filter(|split| *split > 0 && *split < text.len())
}

const fn is_embedding_split_boundary(character: char) -> bool {
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

pub(super) fn mean_embedding_vectors(vectors: &[Vec<f32>], normalize: bool) -> Result<Vec<f32>> {
    let Some(first) = vectors.first() else {
        return Err(AppError::BadRequest(
            "cannot average empty embeddings".to_string(),
        ));
    };
    let dimension = first.len();
    let mut mean = vec![0.0; dimension];
    for vector in vectors {
        if vector.len() != dimension {
            return Err(AppError::Internal(
                "embedding split produced inconsistent vector dimensions".to_string(),
            ));
        }
        for (index, value) in vector.iter().enumerate() {
            mean[index] += value;
        }
    }
    let divisor = f32::from(u16::try_from(vectors.len()).map_err(|_| {
        AppError::Internal("embedding split produced too many vector parts".to_string())
    })?);
    for value in &mut mean {
        *value /= divisor;
    }
    if normalize {
        normalize_l2(&mut mean);
    }
    Ok(mean)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_text_near_middle_handles_cjk_without_whitespace() -> Result<()> {
        let text = "饮用水卫生标准".repeat(10);
        let split = split_text_near_middle(&text)
            .ok_or_else(|| AppError::Internal("missing split point".to_string()))?;
        assert!(split > 0);
        assert!(split < text.len());
        assert!(text.is_char_boundary(split));
        Ok(())
    }

    #[test]
    fn mean_embedding_vectors_renormalizes_split_documents() -> Result<()> {
        let mean = mean_embedding_vectors(&[vec![1.0, 0.0], vec![0.0, 1.0]], true)?;
        let norm = mean.iter().map(|value| value * value).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.0001);
        Ok(())
    }
}
