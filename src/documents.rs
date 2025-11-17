//! Documents search API for RAG (Retrieval-Augmented Generation)
//!
//! Search through uploaded documents and collections to find relevant content.

use crate::proto;

/// Request for document search
#[derive(Debug, Clone)]
pub struct DocumentSearchRequest {
    /// Search query
    pub query: String,
    /// Collection IDs to search in
    pub collection_ids: Vec<String>,
    /// Maximum number of results
    pub limit: Option<i32>,
    /// Ranking metric
    pub ranking_metric: RankingMetric,
    /// Optional search instructions
    pub instructions: Option<String>,
}

/// Ranking metric for search results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RankingMetric {
    /// L2 distance (lower is better)
    L2Distance,
    /// Cosine similarity (higher is better)
    CosineSimilarity,
}

impl DocumentSearchRequest {
    /// Create a new document search request
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            collection_ids: Vec::new(),
            limit: None,
            ranking_metric: RankingMetric::L2Distance,
            instructions: None,
        }
    }

    /// Add a collection ID to search
    pub fn add_collection(mut self, collection_id: impl Into<String>) -> Self {
        self.collection_ids.push(collection_id.into());
        self
    }

    /// Set maximum number of results
    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set ranking metric
    pub fn with_ranking_metric(mut self, metric: RankingMetric) -> Self {
        self.ranking_metric = metric;
        self
    }

    /// Set search instructions
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }
}

/// Response from document search
#[derive(Debug, Clone)]
pub struct DocumentSearchResponse {
    /// Matching document chunks
    pub matches: Vec<SearchMatch>,
}

/// A matching document chunk
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Document file ID
    pub file_id: String,
    /// Chunk ID within the document
    pub chunk_id: String,
    /// Text content of the chunk
    pub content: String,
    /// Relevance score
    pub score: f32,
    /// Collection IDs this document belongs to
    pub collection_ids: Vec<String>,
}

impl From<proto::SearchResponse> for DocumentSearchResponse {
    fn from(proto: proto::SearchResponse) -> Self {
        Self {
            matches: proto.matches.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<proto::SearchMatch> for SearchMatch {
    fn from(proto: proto::SearchMatch) -> Self {
        Self {
            file_id: proto.file_id,
            chunk_id: proto.chunk_id,
            content: proto.chunk_content,
            score: proto.score,
            collection_ids: proto.collection_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_search_request_builder() {
        let request = DocumentSearchRequest::new("quantum computing basics")
            .add_collection("col-1")
            .add_collection("col-2")
            .with_limit(20)
            .with_ranking_metric(RankingMetric::CosineSimilarity)
            .with_instructions("Focus on recent research");

        assert_eq!(request.query, "quantum computing basics");
        assert_eq!(request.collection_ids.len(), 2);
        assert_eq!(request.collection_ids[0], "col-1");
        assert_eq!(request.collection_ids[1], "col-2");
        assert_eq!(request.limit, Some(20));
        assert_eq!(request.ranking_metric, RankingMetric::CosineSimilarity);
        assert_eq!(request.instructions, Some("Focus on recent research".to_string()));
    }

    #[test]
    fn test_document_search_request_minimal() {
        let request = DocumentSearchRequest::new("test query");

        assert_eq!(request.query, "test query");
        assert_eq!(request.collection_ids.len(), 0);
        assert_eq!(request.limit, None);
        assert_eq!(request.ranking_metric, RankingMetric::L2Distance);
        assert_eq!(request.instructions, None);
    }

    #[test]
    fn test_ranking_metric() {
        assert_eq!(RankingMetric::L2Distance, RankingMetric::L2Distance);
        assert_eq!(RankingMetric::CosineSimilarity, RankingMetric::CosineSimilarity);
        assert_ne!(RankingMetric::L2Distance, RankingMetric::CosineSimilarity);
    }

    #[test]
    fn test_search_match_from_proto() {
        let proto_match = proto::SearchMatch {
            file_id: "file-123".to_string(),
            chunk_id: "chunk-456".to_string(),
            chunk_content: "This is the matched content".to_string(),
            score: 0.95,
            collection_ids: vec!["col-1".to_string(), "col-2".to_string()],
        };

        let search_match: SearchMatch = proto_match.into();
        assert_eq!(search_match.file_id, "file-123");
        assert_eq!(search_match.chunk_id, "chunk-456");
        assert_eq!(search_match.content, "This is the matched content");
        assert_eq!(search_match.score, 0.95);
        assert_eq!(search_match.collection_ids.len(), 2);
        assert_eq!(search_match.collection_ids[0], "col-1");
    }

    #[test]
    fn test_document_search_response_from_proto() {
        let proto_response = proto::SearchResponse {
            matches: vec![
                proto::SearchMatch {
                    file_id: "file-1".to_string(),
                    chunk_id: "chunk-1".to_string(),
                    chunk_content: "Content 1".to_string(),
                    score: 0.9,
                    collection_ids: vec!["col-1".to_string()],
                },
                proto::SearchMatch {
                    file_id: "file-2".to_string(),
                    chunk_id: "chunk-2".to_string(),
                    chunk_content: "Content 2".to_string(),
                    score: 0.8,
                    collection_ids: vec!["col-2".to_string()],
                },
            ],
        };

        let response: DocumentSearchResponse = proto_response.into();
        assert_eq!(response.matches.len(), 2);
        assert_eq!(response.matches[0].file_id, "file-1");
        assert_eq!(response.matches[0].content, "Content 1");
        assert_eq!(response.matches[0].score, 0.9);
        assert_eq!(response.matches[1].file_id, "file-2");
    }

    #[test]
    fn test_document_search_request_clone() {
        let request = DocumentSearchRequest::new("query")
            .add_collection("col-1")
            .with_limit(10);

        let cloned = request.clone();
        assert_eq!(cloned.query, request.query);
        assert_eq!(cloned.collection_ids, request.collection_ids);
        assert_eq!(cloned.limit, request.limit);
    }

    #[test]
    fn test_search_match_clone() {
        let search_match = SearchMatch {
            file_id: "file-1".to_string(),
            chunk_id: "chunk-1".to_string(),
            content: "content".to_string(),
            score: 0.9,
            collection_ids: vec!["col-1".to_string()],
        };

        let cloned = search_match.clone();
        assert_eq!(cloned.file_id, search_match.file_id);
        assert_eq!(cloned.chunk_id, search_match.chunk_id);
        assert_eq!(cloned.content, search_match.content);
        assert_eq!(cloned.score, search_match.score);
    }
}
