use anyhow::Result;
use super::types::{PropagationDetailSummary, PropagationFilter};

pub async fn query_propagation_details(
    filter: PropagationFilter,
) -> Result<Vec<PropagationDetailSummary>> {
    // Not implemented in new architecture yet - return empty
    let _ = filter;
    Ok(vec![])
}
