/// `IngestActionSummary` is an enum that represents the successful results of
/// all possible actions that this domain can carry out.
#[derive(Debug, Clone)]
pub enum IngestActionSummary {
    /// `Save` variant represents a save action for a request to save a client
    /// event
    Save(IngestEventSaveSummary),
}

/// `IngestEventSaveSummary` contains information regarding a successful save
/// action.
#[derive(Debug, Clone)]
pub struct IngestEventSaveSummary {
    /// `event_count` is the number of events that were saved in this call.
    pub event_count: usize,
}

impl IngestEventSaveSummary {
    /// `IngestEventSaveSummary` constructor
    pub fn new(event_count: usize) -> Self {
        Self { event_count }
    }
}
