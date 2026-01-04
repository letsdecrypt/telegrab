use serde::Serialize;

pub mod cbz;
pub mod doc;
pub mod pagination;
pub mod pic;

#[derive(Debug, Copy, Clone, Serialize)]
pub struct AffectedRows {
    pub rows_affected: u64,
}
impl AffectedRows {
    pub fn new(rows_affected: u64) -> Self {
        Self { rows_affected }
    }
}
