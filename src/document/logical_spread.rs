// src/document/logical_spread.rs

#[derive(Debug, Clone)]
pub struct LogicalSpread {
    pub left: Option<usize>,
    pub right: Option<usize>,
}
