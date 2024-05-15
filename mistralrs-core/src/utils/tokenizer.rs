use std::{collections::HashMap, path::Path};

use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use tokenizers::Tokenizer;

#[derive(Deserialize)]
struct AddedToken {
    id: usize,
    content: String,
}

/// May fix the tokenizer according to: https://gist.github.com/jneuff/682d47b786329f19291d166957b3274a
pub(crate) fn get_tokenizer<P: AsRef<Path> + Clone>(p: P) -> Result<Tokenizer> {
    let fixed_path = format!("{}_mistralrs_fixed", p.as_ref().display().to_string());
    let fixed_path = Path::new(&fixed_path);

    if !fixed_path.exists() {
        let raw = std::fs::read(p.clone()).map_err(anyhow::Error::msg)?;
        let mut tokenizer: Value = serde_json::from_slice(&raw).unwrap();
        let added_tokens: Vec<AddedToken> =
            serde_json::from_value(tokenizer["added_tokens"].clone()).unwrap();
        let vocab: HashMap<String, usize> =
            serde_json::from_value(tokenizer["model"]["vocab"].clone()).unwrap();
        for token in added_tokens {
            if !vocab.contains_key(&token.content) {
                tokenizer["model"]["vocab"]
                    .as_object_mut()
                    .unwrap()
                    .insert(token.content, token.id.into())
                    .ok_or(())
                    .unwrap_err();
            }
        }
        let raw_fixed = serde_json::to_vec_pretty(&tokenizer).unwrap();
        std::fs::write(fixed_path, raw_fixed).unwrap();
    }

    Tokenizer::from_file(fixed_path).map_err(anyhow::Error::msg)
}
