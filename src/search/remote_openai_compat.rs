use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

use crate::config::RemoteSearchConfig;
use crate::secrets::SecretManager;
use crate::secrets::store::require_secret;

use super::cosine_similarity;
use super::index::{SearchDocument, SearchHit};

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingItem>,
}

#[derive(Debug, Deserialize)]
struct OpenAiEmbeddingItem {
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct RerankResponse {
    data: Vec<RerankItem>,
}

#[derive(Debug, Deserialize)]
struct RerankItem {
    index: usize,
    score: f32,
}

pub async fn search_remote(
    query: &str,
    documents: &[SearchDocument],
    cfg: &RemoteSearchConfig,
    secrets: &SecretManager,
) -> Result<Option<Vec<SearchHit>>> {
    if !cfg.enabled {
        return Ok(None);
    }

    let base_url = match &cfg.base_url {
        Some(url) => url.trim_end_matches('/').to_string(),
        None => return Ok(None),
    };

    let api_key = match &cfg.api_key_secret {
        Some(secret_key) => require_secret(secrets.store(), secrets.resolvers(), secret_key)?,
        None => return Ok(None),
    };

    let client = Client::builder()
        .timeout(std::time::Duration::from_millis(cfg.timeout_ms))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .context("failed building remote search client")?;

    let mut texts = Vec::with_capacity(documents.len() + 1);
    texts.push(query.to_string());
    texts.extend(documents.iter().map(|d| d.text.clone()));

    let embeddings_url = format!("{base_url}{}", cfg.embeddings_path);
    let embed_response = client
        .post(&embeddings_url)
        .bearer_auth(&api_key)
        .json(&json!({
            "model": "text-embedding-3-small",
            "input": texts,
        }))
        .send()
        .await
        .with_context(|| format!("remote embeddings request failed: {embeddings_url}"))?;

    if !embed_response.status().is_success() {
        return Ok(None);
    }

    let embeddings = embed_response
        .json::<OpenAiEmbeddingResponse>()
        .await
        .context("failed decoding remote embeddings response")?;

    if embeddings.data.len() != documents.len() + 1 {
        return Ok(None);
    }

    let query_embedding = &embeddings.data[0].embedding;
    let mut scored: Vec<(usize, f32)> = embeddings
        .data
        .iter()
        .enumerate()
        .skip(1)
        .map(|(idx, emb)| (idx - 1, cosine_similarity(query_embedding, &emb.embedding)))
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));

    let rerank_url = format!("{base_url}{}", cfg.rerank_path);
    let rerank_docs: Vec<String> = scored
        .iter()
        .take(40)
        .map(|(idx, _)| documents[*idx].text.clone())
        .collect();

    let reranked = client
        .post(&rerank_url)
        .bearer_auth(&api_key)
        .json(&json!({
            "query": query,
            "documents": rerank_docs,
            "top_n": 10,
        }))
        .send()
        .await;

    let mut hits = Vec::new();

    if let Ok(resp) = reranked
        && resp.status().is_success()
        && let Ok(parsed) = resp.json::<RerankResponse>().await
    {
        let top_indices: Vec<usize> = scored.iter().take(40).map(|(idx, _)| *idx).collect();
        for item in parsed.data.into_iter().take(10) {
            if let Some(doc_idx) = top_indices.get(item.index) {
                let doc = &documents[*doc_idx];
                hits.push(SearchHit {
                    key: doc.key.clone(),
                    score: item.score,
                    summary: doc.summary.clone(),
                });
            }
        }
        if !hits.is_empty() {
            return Ok(Some(hits));
        }
    }

    for (idx, score) in scored.into_iter().take(10) {
        let doc = &documents[idx];
        hits.push(SearchHit {
            key: doc.key.clone(),
            score,
            summary: doc.summary.clone(),
        });
    }

    Ok(Some(hits))
}
