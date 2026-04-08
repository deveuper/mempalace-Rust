#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]
#![allow(unused_must_use)]

use chrono::{DateTime, Utc};

/// Search filters
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Filter by wing
    pub wing: Option<String>,
    /// Filter by room
    pub room: Option<String>,
    /// Filter by hall
    pub hall: Option<String>,
    /// Filter by date range (start)
    pub date_from: Option<DateTime<Utc>>,
    /// Filter by date range (end)
    pub date_to: Option<DateTime<Utc>>,
    /// Filter by source file
    pub source_file: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Minimum similarity score
    pub min_score: Option<f32>,
}

impl SearchFilters {
    /// Create new empty filters
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by wing
    pub fn with_wing(mut self, wing: impl Into<String>) -> Self {
        self.wing = Some(wing.into());
        self
    }

    /// Filter by room
    pub fn with_room(mut self, room: impl Into<String>) -> Self {
        self.room = Some(room.into());
        self
    }

    /// Filter by hall
    pub fn with_hall(mut self, hall: impl Into<String>) -> Self {
        self.hall = Some(hall.into());
        self
    }

    /// Filter by date range
    pub fn with_date_range(
        mut self,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Self {
        self.date_from = from;
        self.date_to = to;
        self
    }

    /// Filter by source file
    pub fn with_source_file(mut self, source: impl Into<String>) -> Self {
        self.source_file = Some(source.into());
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set minimum score
    pub fn with_min_score(mut self, score: f32) -> Self {
        self.min_score = Some(score);
        self
    }
}
