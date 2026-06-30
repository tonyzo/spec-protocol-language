//! 数据模型 —— SPEC Protocol 核心数据结构
//!
//! 参见 CONTEXT.md 与 ADR-001/003/005。
//! SPEC 不做数值计算，所有值与限值由外部计算软件提供。

use std::collections::HashMap;

use crate::rule::SpecRule;

/// 带单位的量。参数值与限值共用此结构。
/// 参见 ADR-005：unit 一致性由 UnitCheck 校验，不静默换算。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
}

/// 参数表（分区组织，ADR-005）。
/// - `parameters`: 数值型设计结果（计算软件输出），如壁厚、应力。
/// - `limits`: 数值型判据边界值（同样由计算软件提供），如最小厚度、许用应力。
/// - `attrs`: 分类属性（字符串），如元件类型、材质牌号。用于适用条件筛选。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ParameterTable {
    pub parameters: HashMap<String, Quantity>,
    pub limits: HashMap<String, Quantity>,
    pub attrs: HashMap<String, String>,
}

/// 元数据：标识被校核的设备/元件（用于报告溯源）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct DesignContext {
    pub element_id: Option<String>,      // 如 "vessel.V-101.shell"
    pub element_type: Option<String>,    // 如 "内压圆筒"
    pub standard: Option<String>,        // 如 "GB/T 150.3-2024"
}

/// 单条规则的校核结果。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuleResult {
    pub rule_id: String,
    pub clause: String,             // GB150 条款号，如 "5.3"
    pub status: CheckStatus,
    pub severity: Severity,
    pub message: String,
    pub violations: Vec<Violation>, // 空表示通过
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pass,
    Fail,
    /// 规则不适用（applicability 未命中）
    NotApplicable,
    /// 缺少参数，无法判定
    Skipped,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// 违规：一条未满足的断言。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Violation {
    pub rule_id: String,
    pub clause: String,
    /// 断言的人类可读描述，如 "delta >= delta_min"
    pub assertion_desc: String,
    pub actual: Option<f64>,
    pub expected: Option<f64>,
    pub unit: Option<String>,
    /// 偏差量 = actual - expected（同量纲时有意义）
    pub deviation: Option<f64>,
}

/// 合规报告（中性结构，由 ReportWriter trait 转为目标格式）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComplianceReport {
    pub context: DesignContext,
    pub results: Vec<RuleResult>,
    pub summary: ReportSummary,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ReportSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub not_applicable: usize,
    pub skipped: usize,
    pub errors: usize,
    pub warnings: usize,
}

impl ComplianceReport {
    pub fn summarize(&mut self) {
        let mut s = ReportSummary::default();
        for r in &self.results {
            s.total += 1;
            match r.status {
                CheckStatus::Pass => s.passed += 1,
                CheckStatus::Fail => {
                    s.failed += 1;
                    if r.severity == Severity::Error {
                        s.errors += 1;
                    } else if r.severity == Severity::Warning {
                        s.warnings += 1;
                    }
                }
                CheckStatus::NotApplicable => s.not_applicable += 1,
                CheckStatus::Skipped => s.skipped += 1,
            }
        }
        self.summary = s;
    }
}

// ──── 审查点生命周期数据结构 (ADR-006) ────

/// 候选审查点的匹配状态。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CandidateMatchStatus {
    /// applicability 命中，强候选
    Applicable,
    /// applicability 缺参数，弱候选（需人工判断）
    Uncertain,
    /// 无 applicability 定义，默认候选（规则始终适用）
    NoApplicability,
}

/// 候选审查点：DiscoveryEngine.discover() 的输出。
/// 每条候选包含规则本身、匹配状态和推荐理由。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CandidateReviewPoint {
    pub rule: SpecRule,
    pub match_status: CandidateMatchStatus,
    /// 推荐理由，如 "applicability 命中: element_type=内压圆筒"
    pub reason: String,
}

/// 注册表条目：记录一个已沉淀的审查点文件。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryEntry {
    /// 标准号，如 "GB150.3"
    pub standard: String,
    /// 条款号，如 "5.3"
    pub clause: String,
    /// 文件路径（相对于 confirmed 目录），如 "内压圆筒/GB150.3-5.3-2024.yaml"
    pub file_path: String,
    /// 规则数量
    pub rule_count: usize,
    /// 确认时间（ISO 8601）
    pub confirmed_at: String,
}

/// 审查点注册表：映射设计类型 → 审查点文件列表。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct ReviewPointRegistry {
    /// 注册表版本
    pub version: String,
    /// 设计类型 → 注册条目列表
    pub design_types: HashMap<String, Vec<RegistryEntry>>,
}

impl ReviewPointRegistry {
    /// 检查设计类型是否已有审查点集合
    pub fn has_design_type(&self, design_type: &str) -> bool {
        self.design_types.contains_key(design_type)
    }

    /// 获取设计类型的审查点文件列表
    pub fn get_entries(&self, design_type: &str) -> Option<&[RegistryEntry]> {
        self.design_types.get(design_type).map(|v| v.as_slice())
    }

    /// 添加或更新设计类型的注册条目
    pub fn add_entry(&mut self, design_type: &str, entry: RegistryEntry) {
        self.design_types
            .entry(design_type.to_string())
            .or_default()
            .push(entry);
    }
}
