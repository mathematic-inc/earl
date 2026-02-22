use anyhow::{Context, Result};
use fastembed::{
    EmbeddingModel, RerankInitOptions, RerankerModel, TextEmbedding, TextInitOptions, TextRerank,
};

use crate::config::SearchConfig;

use super::cosine_similarity;
use super::index::{SearchDocument, SearchHit};

pub fn search_local(
    query: &str,
    documents: &[SearchDocument],
    cfg: &SearchConfig,
) -> Result<Vec<SearchHit>> {
    if documents.is_empty() {
        return Ok(Vec::new());
    }

    let embedding_model: EmbeddingModel = cfg
        .local
        .embedding_model
        .clone()
        .try_into()
        .map_err(|err: String| anyhow::anyhow!(err))
        .with_context(|| format!("invalid embedding model `{}`", cfg.local.embedding_model))?;

    let reranker_model: RerankerModel = cfg
        .local
        .reranker_model
        .clone()
        .try_into()
        .map_err(|err: String| anyhow::anyhow!(err))
        .with_context(|| format!("invalid reranker model `{}`", cfg.local.reranker_model))?;

    let mut embedder = TextEmbedding::try_new(
        TextInitOptions::new(embedding_model).with_show_download_progress(false),
    )
    .context("failed to initialize local embedding model")?;

    let mut reranker = TextRerank::try_new(
        RerankInitOptions::new(reranker_model).with_show_download_progress(false),
    )
    .context("failed to initialize local reranker model")?;

    let doc_texts: Vec<String> = documents.iter().map(|d| d.text.clone()).collect();
    let query_text = vec![query.to_string()];

    let query_embedding = embedder
        .embed(query_text, None)
        .context("failed generating query embedding")?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("embedding model returned empty query embedding"))?;

    let doc_embeddings = embedder
        .embed(doc_texts.clone(), None)
        .context("failed generating document embeddings")?;

    let mut scored: Vec<(usize, f32)> = doc_embeddings
        .iter()
        .enumerate()
        .map(|(idx, emb)| (idx, cosine_similarity(&query_embedding, emb)))
        .collect();

    scored.sort_by(|a, b| b.1.total_cmp(&a.1));
    let top_k = cfg.top_k.min(scored.len());
    let top_indices: Vec<usize> = scored.into_iter().take(top_k).map(|(idx, _)| idx).collect();

    let rerank_docs: Vec<String> = top_indices
        .iter()
        .map(|idx| doc_texts[*idx].clone())
        .collect();

    let reranked = reranker
        .rerank(query.to_string(), rerank_docs, false, None)
        .context("local reranking failed")?;

    let mut hits = Vec::new();
    for rr in reranked.into_iter().take(cfg.rerank_k) {
        if let Some(doc_idx) = top_indices.get(rr.index) {
            let doc = &documents[*doc_idx];
            hits.push(SearchHit {
                key: doc.key.clone(),
                score: rr.score,
                summary: doc.summary.clone(),
            });
        }
    }

    Ok(hits)
}
