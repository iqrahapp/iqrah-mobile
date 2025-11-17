// semantic/mod.rs
// Semantic similarity grading using model2vec with BGE-M3 embeddings

pub mod embedding;
pub mod grader;

#[cfg(test)]
mod tests;

pub use embedding::SemanticEmbedder;
pub use grader::{SemanticGrade, SemanticGradeLabel, SemanticGrader, SEMANTIC_EMBEDDER};
