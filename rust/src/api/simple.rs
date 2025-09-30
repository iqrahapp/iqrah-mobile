use anyhow::Result;

use crate::{
    app::app,
    repository::{PropagationDetailRecord, PropagationQueryOptions},
};

use super::types::{PropagationDetailSummary, PropagationFilter};

fn map_record_to_summary(record: PropagationDetailRecord) -> PropagationDetailSummary {
    PropagationDetailSummary {
        event_timestamp: record.event_timestamp,
        source_node_text: record.source_text.unwrap_or(record.source_node_id),
        target_node_text: record.target_text.unwrap_or(record.target_node_id),
        energy_change: record.energy_change,
        path: record.path,
        reason: record.reason,
    }
}

pub async fn query_propagation_details(
    filter: PropagationFilter,
) -> Result<Vec<PropagationDetailSummary>> {
    let normalized_limit = if filter.limit == 0 { 50 } else { filter.limit };
    let options = PropagationQueryOptions {
        start_time_secs: filter.start_time_secs,
        end_time_secs: filter.end_time_secs,
        limit: normalized_limit,
    };

    let records = app().service.query_propagation_details(options).await?;

    Ok(records.into_iter().map(map_record_to_summary).collect())
}
