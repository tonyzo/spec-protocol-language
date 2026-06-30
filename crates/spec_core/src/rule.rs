//! 规则定义 —— 从 spec_schema re-export
//!
//! 类型定义已移至 spec_schema crate，
//! 确保 spec_core 与 term_extractor 共享同一类型契约。

pub use spec_schema::{Assertion, Operand, SpecRule};
