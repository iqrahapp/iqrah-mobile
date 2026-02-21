use crate::{domain::ImportStats, ContentRepository};
use anyhow::Result;
use std::{io::Read, sync::Arc};

/// Import knowledge graph from CBOR bytes
pub async fn import_cbor_graph_from_bytes<R>(
    _repo: Arc<dyn ContentRepository>,
    _reader: R,
) -> Result<ImportStats>
where
    R: Read + Send + 'static,
{
    Err(anyhow::anyhow!(
        "CBOR graph import fallback is disabled: parsed data is not persisted in the current runtime. \
Use the bundled content database path and pass empty kg_bytes."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockContentRepository;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_cbor_import_fail_fast_is_explicit() {
        let repo = Arc::new(MockContentRepository::new());
        let err = import_cbor_graph_from_bytes(repo, Cursor::new(Vec::<u8>::new()))
            .await
            .expect_err("CBOR fallback must fail fast when persistence is disabled");

        let msg = err.to_string();
        assert!(msg.contains("disabled"));
        assert!(msg.contains("not persisted"));
    }
}
