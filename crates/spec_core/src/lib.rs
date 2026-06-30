//! SPEC Protocol Core — 合规引擎核心库
//!
//! 参见 CONTEXT.md 和 docs/adr/ 下的 ADR-001 ~ ADR-006。
//!
//! 模块结构：
//! - `model`  — 数据模型 (ParameterTable / ComplianceReport / ...)
//! - `rule`   — 规则定义 (SpecRule / Assertion / Operand)
//! - `assertion` — 断言求值器 (比较 + 布尔运算 + 量纲校验)
//! - `engine` — 合规引擎主流程 (ComplianceEngine)
//! - `report` — 报告输出适配器 (ReportWriter trait)
//! - `registry` — 规则库 + 审查点注册表 (RuleLibrary / ReviewPointRegistry)
//! - `discovery` — 审查点发现引擎 (DiscoveryEngine)

pub mod assertion;
pub mod discovery;
pub mod engine;
pub mod model;
pub mod registry;
pub mod report;
pub mod rule;

// 顶层 re-export，方便外部使用
pub use assertion::{eval, EvalOutcome, SpecError};
pub use discovery::{DiscoveryEngine, DiscoveryError};
pub use engine::ComplianceEngine;
pub use model::{
    CandidateMatchStatus, CandidateReviewPoint, CheckStatus, ComplianceReport, DesignContext,
    ParameterTable, Quantity, RegistryEntry, ReportSummary, ReviewPointRegistry, RuleResult,
    Severity, Violation,
};
pub use registry::{RegistryError, RuleLibrary};
pub use report::{JsonWriter, ReportError, ReportWriter, RvmWriter};
pub use rule::{Assertion, Operand, SpecRule};
