pub mod dto;
pub mod entity;

use async_graphql::Enum;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}
pub struct PaginationArgs {
    pub cursor: Option<i32>,
    pub limit: usize,
    pub direction: Direction,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
pub enum SortOrder {
    Asc,
    Desc,
}
