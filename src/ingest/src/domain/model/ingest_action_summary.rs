#[derive(Debug, Clone)]
pub struct IngestEventSaveSummary {
    pub event_count: usize,
}

#[derive(Debug, Clone)]
pub enum IngestActionSummary {
    Save(IngestEventSaveSummary),
}
