//! SPEC Protocol 共享数据契约
//!
//! 此 crate 是 spec_core 和 term_extractor 之间的类型契约权威源。
//! 两个组件都依赖此 crate，确保生成的 YAML 可被正确反序列化。
//!
//! 共享类型：
//! - `ParamType` — 参数分类（parameter/limit/attr）
//! - `Severity` — 规则严重度（error/warning/info）
//! - `Operand` / `Assertion` / `SpecRule` — 规则定义与断言判定树

pub mod common;
pub mod rule;

pub use common::{ParamType, Severity};
pub use rule::{Assertion, Operand, SpecRule};
