//! 冲突检测模块
//!
//! ConflictDetector 是一个深模块：一次构建索引，多种冲突检测策略复用。
//! 替代了原来四个浅模块(terminology.rs/parameter.rs/rule.rs/unit.rs)。

pub mod report;

use serde::{Deserialize, Serialize};
use crate::models::{ParamMapping, TermEntry};
use std::collections::HashMap;

// ── 类型定义 ──────────────────────────────────

/// 冲突严重度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictSeverity {
    Info,
    Warning,
    Error,
}

/// 冲突类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    Terminology,
    Parameter,
    Rule,
    Unit,
}

impl ConflictType {
    pub fn as_str(&self) -> &str {
        match self {
            ConflictType::Terminology => "术语冲突",
            ConflictType::Parameter => "参数冲突",
            ConflictType::Rule => "规则冲突",
            ConflictType::Unit => "单位冲突",
        }
    }
}

/// 冲突项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictItem {
    pub conflict_type: ConflictType,
    pub severity: ConflictSeverity,
    pub standards: Vec<String>,
    pub details: String,
    pub resolution: String,
}

/// 冲突检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResult {
    pub total_conflicts: usize,
    pub terminology_conflicts: usize,
    pub parameter_conflicts: usize,
    pub rule_conflicts: usize,
    pub unit_conflicts: usize,
    pub conflicts: Vec<ConflictItem>,
}

impl ConflictResult {
    pub fn new() -> Self {
        ConflictResult {
            total_conflicts: 0,
            terminology_conflicts: 0,
            parameter_conflicts: 0,
            rule_conflicts: 0,
            unit_conflicts: 0,
            conflicts: Vec::new(),
        }
    }

    pub fn add_conflict(&mut self, item: ConflictItem) {
        match item.conflict_type {
            ConflictType::Terminology => self.terminology_conflicts += 1,
            ConflictType::Parameter => self.parameter_conflicts += 1,
            ConflictType::Rule => self.rule_conflicts += 1,
            ConflictType::Unit => self.unit_conflicts += 1,
        }
        self.total_conflicts += 1;
        self.conflicts.push(item);
    }
}

// ── ConflictDetector 深模块 ────────────────────

/// 冲突检测器 — 一次构建索引，多次检测复用
///
/// 替代原来四个独立浅模块(terminology.rs/parameter.rs/rule.rs/unit.rs)，
/// 将"按 param_name 分组"的公共逻辑集中到 new() 中，
/// 每种冲突检测变为针对同一索引的不同策略。
pub struct ConflictDetector<'a> {
    /// 预构建索引: param_name -> [(standard_name, term_entry)]
    index: HashMap<String, Vec<(&'a str, &'a TermEntry)>>,
}

impl<'a> ConflictDetector<'a> {
    /// 从多个标准的参数映射构建检测器（索引构建一次）
    pub fn new(mappings: &[(&'a str, &'a ParamMapping)]) -> Self {
        let mut index: HashMap<String, Vec<(&str, &TermEntry)>> = HashMap::new();

        for (standard_name, mapping) in mappings {
            for term in &mapping.terms {
                let param_name = term.spec_mapping.name.clone();
                index
                    .entry(param_name)
                    .or_insert_with(Vec::new)
                    .push((standard_name, term));
            }
        }

        ConflictDetector { index }
    }

    /// 运行所有冲突检测，返回结果
    pub fn detect_all(&self) -> ConflictResult {
        let mut result = ConflictResult::new();
        self.detect_terminology(&mut result);
        self.detect_parameter(&mut result);
        self.detect_unit(&mut result);
        result
    }

    /// 术语冲突: 同一参数名对应不同标准术语 (Info)
    fn detect_terminology(&self, result: &mut ConflictResult) {
        for (param_name, entries) in &self.index {
            if entries.len() < 2 {
                continue;
            }

            let unique_terms: std::collections::HashSet<_> =
                entries.iter().map(|(_, t)| t.standard_term.as_str()).collect();

            if unique_terms.len() > 1 {
                let standards: Vec<String> = entries.iter().map(|(s, _)| s.to_string()).collect();
                let details = format!(
                    "参数 '{}' 在不同标准中有不同术语: {}",
                    param_name,
                    entries.iter()
                        .map(|(s, t)| format!("{}: {}", s, t.standard_term))
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                result.add_conflict(ConflictItem {
                    conflict_type: ConflictType::Terminology,
                    severity: ConflictSeverity::Info,
                    standards,
                    details,
                    resolution: "已映射到同一SPEC参数,无需处理".to_string(),
                });
            }
        }
    }

    /// 参数冲突: 同一参数名对应不同定义 (Warning)
    fn detect_parameter(&self, result: &mut ConflictResult) {
        for (param_name, entries) in &self.index {
            if entries.len() < 2 {
                continue;
            }

            let unique_defs: std::collections::HashSet<_> =
                entries.iter().map(|(_, t)| t.definition.as_str()).collect();

            if unique_defs.len() > 1 {
                let standards: Vec<String> = entries.iter().map(|(s, _)| s.to_string()).collect();
                let details = format!(
                    "参数 '{}' 在不同标准中有不同定义:\n{}",
                    param_name,
                    entries.iter()
                        .map(|(s, t)| format!("  {} ({}): {}", s, t.standard_term, t.definition))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                result.add_conflict(ConflictItem {
                    conflict_type: ConflictType::Parameter,
                    severity: ConflictSeverity::Warning,
                    standards,
                    details,
                    resolution: "定义不同,需要人工确认哪个定义更准确".to_string(),
                });
            }
        }
    }

    /// 单位冲突: 同一参数名对应不同单位 (Warning)
    /// 注: 原 rule.rs 与 unit.rs 检测逻辑完全相同,已合并为此方法。
    /// 真正的规则冲突(比较 assertion)需要解析 YAML,超出当前范围。
    fn detect_unit(&self, result: &mut ConflictResult) {
        for (param_name, entries) in &self.index {
            if entries.len() < 2 {
                continue;
            }

            let unique_units: std::collections::HashSet<_> =
                entries.iter().map(|(_, t)| t.spec_mapping.unit.as_deref()).collect();

            if unique_units.len() > 1 {
                let standards: Vec<String> = entries.iter().map(|(s, _)| s.to_string()).collect();
                let details = format!(
                    "参数 '{}' 在不同标准中使用不同单位:\n{}",
                    param_name,
                    entries.iter()
                        .map(|(s, t)| format!("  {}: {:?}", s, t.spec_mapping.unit))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                let resolution = generate_unit_conversion_advice(param_name);

                result.add_conflict(ConflictItem {
                    conflict_type: ConflictType::Unit,
                    severity: ConflictSeverity::Warning,
                    standards,
                    details,
                    resolution,
                });
            }
        }
    }
}

/// 生成单位转换建议
fn generate_unit_conversion_advice(param_name: &str) -> String {
    let mut advice = String::from("需要单位转换: ");

    if param_name.contains("pressure") {
        advice.push_str("1 MPa = 145.038 psi");
    } else if param_name.contains("thickness") || param_name.contains("diameter") {
        advice.push_str("1 in = 25.4 mm");
    } else if param_name.contains("temperature") {
        advice.push_str("°F = °C × 9/5 + 32");
    } else if param_name.contains("stress") {
        advice.push_str("1 MPa = 145.038 psi");
    } else {
        advice.push_str("请参考标准附录中的单位换算表");
    }

    advice
}
