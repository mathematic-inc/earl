use serde::{Deserialize, Serialize};

use crate::template::catalog::{TemplateCatalog, TemplateScope};
#[allow(unused_imports)]
use crate::template::schema::OperationProtocol;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub key: String,
    pub text: String,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub categories: Vec<String>,
    pub mode: String,
    pub source_scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub key: String,
    pub score: f32,
    pub summary: String,
}

pub fn build_documents(catalog: &TemplateCatalog) -> Vec<SearchDocument> {
    catalog
        .values()
        .map(|entry| {
            let mut corpus = vec![
                entry.key.clone(),
                entry.title.clone(),
                entry.summary.clone(),
                entry.description.clone(),
            ];
            corpus.extend(entry.categories.iter().cloned());
            #[allow(unreachable_patterns)]
            match entry.template.operation.protocol() {
                #[cfg(feature = "http")]
                OperationProtocol::Http => {
                    if let Some(url) = entry.template.operation.request_url() {
                        corpus.push(url.to_string());
                    }
                }
                #[cfg(feature = "graphql")]
                OperationProtocol::Graphql => {
                    if let Some(url) = entry.template.operation.request_url() {
                        corpus.push(url.to_string());
                    }
                }
                #[cfg(feature = "grpc")]
                OperationProtocol::Grpc => {
                    if let Some(url) = entry.template.operation.request_url() {
                        corpus.push(url.to_string());
                    }
                    if let Some((service, method)) = entry.template.operation.grpc_service_method()
                    {
                        corpus.push(format!("{service}.{method}"));
                    }
                }
                #[cfg(feature = "bash")]
                OperationProtocol::Bash => {
                    if let Some(script) = entry.template.operation.bash_script() {
                        corpus.push(script.to_string());
                    }
                }
                #[cfg(feature = "sql")]
                OperationProtocol::Sql => {
                    if let Some(query) = entry.template.operation.sql_query() {
                        corpus.push(query.to_string());
                    }
                }
                _ => {}
            }
            corpus.push(entry.template.result.output.clone());
            for param in &entry.template.params {
                corpus.push(format!("{}:{}", param.name, param.r#type));
            }

            SearchDocument {
                key: entry.key.clone(),
                text: corpus.join("\n"),
                title: entry.title.clone(),
                summary: entry.summary.clone(),
                description: entry.description.clone(),
                categories: entry.categories.clone(),
                mode: entry.mode.as_str().to_string(),
                source_scope: match entry.source.scope {
                    TemplateScope::Local => "local".to_string(),
                    TemplateScope::Global => "global".to_string(),
                },
            }
        })
        .collect()
}
