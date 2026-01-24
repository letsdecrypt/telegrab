pub mod dto;
pub mod entity;


#[derive(Debug,PartialOrd, PartialEq)]
pub enum Direction {
    Forward,
    Backward,
}
pub struct PaginationArgs {
   pub cursor: Option<i32>,
   pub limit: usize,
   pub direction: Direction,
}
