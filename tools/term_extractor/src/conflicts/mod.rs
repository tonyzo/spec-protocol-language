//! 冲突检测模块
//!
//! ConflictDetector 是一个深模块：一次构建索引，多种冲突检测策略复用。
//! 替代了原来四个浅模块(terminology.rs/parameter.rs/rule.rs/unit.rs)。

pub mod report;

use serde::{Deserialize, Serialize};
use crate::models::{ParamMapping, TermEntry};
use spec_schema::SpecRule;
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

/// 规则冲突检测：比较相同 rule_id 的规则 assertion 差异
///
/// 接收多个标准的规则列表，查找相同 rule_id 但 assertion 描述不同的规则。
/// 用于发现同一场景在不同标准中要求不同的情况。
pub fn detect_rule_conflicts(
    standards: &[(&str, &[SpecRule])],
    result: &mut ConflictResult,
) {
    // 按 rule_id 分组
    let mut rule_map: HashMap<String, Vec<(&str, &SpecRule)>> = HashMap::new();

    for (standard_name, rules) in standards {
        for rule in *rules {
            rule_map
                .entry(rule.id.clone())
                .or_insert_with(Vec::new)
                .push((standard_name, rule));
        }
    }

    // 查找相同 rule_id 但 assertion 不同的规则
    for (rule_id, entries) in &rule_map {
        if entries.len() < 2 {
            continue;
        }

        // 比较 assertion 描述
        let unique_assertions: std::collections::HashSet<_> =
            entries.iter().map(|(_, r)| r.describe_assertion()).collect();

        if unique_assertions.len() > 1 {
            let std_names: Vec<String> = entries.iter().map(|(s, _)| s.to_string()).collect();
            let details = format!(
                "规则 '{}' 在不同标准中有不同判据:\n{}",
                rule_id,
                entries.iter()
                    .map(|(s, r)| format!("  {}: {}", s, r.describe_assertion()))
                    .collect::<Vec<_>>()
                    .join("\n")
            );

            result.add_conflict(ConflictItem {
                conflict_type: ConflictType::Rule,
                severity: ConflictSeverity::Error,
                standards: std_names,
                details,
                resolution: "规则判据不同,需要人工确认哪个标准的要求更严格".to_string(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SpecMapping, Confidence};
    use spec_schema::{Assertion, Operand, ParamType, Severity};

    fn make_term(
        term_id: &str,
        standard_term: &str,
        definition: &str,
        param_name: &str,
        unit: Option<&str>,
    ) -> TermEntry {
        TermEntry {
            term_id: term_id.to_string(),
            standard_term: standard_term.to_string(),
            definition: definition.to_string(),
            spec_mapping: SpecMapping {
                param_type: ParamType::Parameter,
                name: param_name.to_string(),
                unit: unit.map(|s| s.to_string()),
                data_type: "f64".to_string(),
                allowed_values: None,
            },
            source_clause: "5.3".to_string(),
            referenced_by: vec![],
            related_terms: vec![],
            confidence: Confidence::High,
            note: None,
            synonyms: vec![],
        }
    }

    fn make_mapping(standard: &str, terms: Vec<TermEntry>) -> ParamMapping {
        let mut mapping = ParamMapping::new(standard, "2024", vec!["1".to_string()]);
        for term in terms {
            mapping.add_term(term);
        }
        mapping
    }

    #[test]
    fn test_no_conflict_single_standard() {
        let mapping = make_mapping("GB150", vec![
            make_term("pressure.design", "设计压力", "压力容器顶部压力", "pressure_design", Some("MPa")),
        ]);
        let detector = ConflictDetector::new(&[("GB150", &mapping)]);
        let result = detector.detect_all();
        assert_eq!(result.total_conflicts, 0);
    }

    #[test]
    fn test_no_conflict_same_values() {
        let mapping_a = make_mapping("GB150", vec![
            make_term("pressure.design", "设计压力", "压力容器顶部压力", "pressure_design", Some("MPa")),
        ]);
        let mapping_b = make_mapping("NB47012", vec![
            make_term("pressure.design", "设计压力", "压力容器顶部压力", "pressure_design", Some("MPa")),
        ]);
        let detector = ConflictDetector::new(&[("GB150", &mapping_a), ("NB47012", &mapping_b)]);
        let result = detector.detect_all();
        assert_eq!(result.total_conflicts, 0);
    }

    #[test]
    fn test_terminology_conflict() {
        let mapping_a = make_mapping("GB150", vec![
            make_term("pressure.design", "设计压力", "压力容器顶部压力", "pressure_design", Some("MPa")),
        ]);
        let mapping_b = make_mapping("ASME", vec![
            make_term("pressure.design", "Design Pressure", "压力容器顶部压力", "pressure_design", Some("MPa")),
        ]);
        let detector = ConflictDetector::new(&[("GB150", &mapping_a), ("ASME", &mapping_b)]);
        let result = detector.detect_all();
        assert_eq!(result.terminology_conflicts, 1);
        assert_eq!(result.total_conflicts, 1);
    }

    #[test]
    fn test_parameter_conflict() {
        let mapping_a = make_mapping("GB150", vec![
            make_term("pressure.design", "设计压力", "压力容器顶部压力", "pressure_design", Some("MPa")),
        ]);
        let mapping_b = make_mapping("NB47012", vec![
            make_term("pressure.design", "设计压力", "容器顶部表压", "pressure_design", Some("MPa")),
        ]);
        let detector = ConflictDetector::new(&[("GB150", &mapping_a), ("NB47012", &mapping_b)]);
        let result = detector.detect_all();
        assert_eq!(result.parameter_conflicts, 1);
    }

    #[test]
    fn test_unit_conflict() {
        let mapping_a = make_mapping("GB150", vec![
            make_term("pressure.design", "设计压力", "压力容器顶部压力", "pressure_design", Some("MPa")),
        ]);
        let mapping_b = make_mapping("ASME", vec![
            make_term("pressure.design", "设计压力", "压力容器顶部压力", "pressure_design", Some("psi")),
        ]);
        let detector = ConflictDetector::new(&[("GB150", &mapping_a), ("ASME", &mapping_b)]);
        let result = detector.detect_all();
        assert_eq!(result.unit_conflicts, 1);
    }

    #[test]
    fn test_multiple_conflicts_same_param() {
        let mapping_a = make_mapping("GB150", vec![
            make_term("pressure.design", "设计压力", "定义A", "pressure_design", Some("MPa")),
        ]);
        let mapping_b = make_mapping("ASME", vec![
            make_term("pressure.design", "Design Pressure", "定义B", "pressure_design", Some("psi")),
        ]);
        let detector = ConflictDetector::new(&[("GB150", &mapping_a), ("ASME", &mapping_b)]);
        let result = detector.detect_all();
        assert_eq!(result.terminology_conflicts, 1);
        assert_eq!(result.parameter_conflicts, 1);
        assert_eq!(result.unit_conflicts, 1);
        assert_eq!(result.total_conflicts, 3);
    }

    #[test]
    fn test_detect_rule_conflicts_same_assertion() {
        let rule = SpecRule {
            id: "sigma_check".to_string(),
            clause: "5.3".to_string(),
            severity: Severity::Error,
            applicability: None,
            assertion: Assertion::Le(
                Operand::ParamRef { param: "sigma".into() },
                Operand::LimitRef { limit: "sigma_allow".into() },
            ),
            message: None,
        };
        let rules_a = vec![rule.clone()];
        let rules_b = vec![rule];
        let mut result = ConflictResult::new();
        detect_rule_conflicts(&[("GB150", &rules_a), ("NB47012", &rules_b)], &mut result);
        assert_eq!(result.rule_conflicts, 0);
    }

    #[test]
    fn test_detect_rule_conflicts_different_assertion() {
        let rule_a = SpecRule {
            id: "sigma_check".to_string(),
            clause: "5.3".to_string(),
            severity: Severity::Error,
            applicability: None,
            assertion: Assertion::Le(
                Operand::ParamRef { param: "sigma".into() },
                Operand::LimitRef { limit: "sigma_allow".into() },
            ),
            message: None,
        };
        let rule_b = SpecRule {
            id: "sigma_check".to_string(),
            clause: "5.3".to_string(),
            severity: Severity::Error,
            applicability: None,
            assertion: Assertion::Lt(
                Operand::ParamRef { param: "sigma".into() },
                Operand::LimitRef { limit: "sigma_allow".into() },
            ),
            message: None,
        };
        let rules_a = vec![rule_a];
        let rules_b = vec![rule_b];
        let mut result = ConflictResult::new();
        detect_rule_conflicts(&[("GB150", &rules_a), ("ASME", &rules_b)], &mut result);
        assert_eq!(result.rule_conflicts, 1);
    }

    #[test]
    fn test_detect_rule_conflicts_no_matching_id() {
        let rule_a = SpecRule {
            id: "rule_a".to_string(),
            clause: "5.3".to_string(),
            severity: Severity::Error,
            applicability: None,
            assertion: Assertion::Exists(Operand::AttrRef { attr: "pwht".into() }),
            message: None,
        };
        let rule_b = SpecRule {
            id: "rule_b".to_string(),
            clause: "7.3".to_string(),
            severity: Severity::Error,
            applicability: None,
            assertion: Assertion::Exists(Operand::AttrRef { attr: "rt".into() }),
            message: None,
        };
        let rules_a = vec![rule_a];
        let rules_b = vec![rule_b];
        let mut result = ConflictResult::new();
        detect_rule_conflicts(&[("GB150", &rules_a), ("ASME", &rules_b)], &mut result);
        assert_eq!(result.rule_conflicts, 0);
    }

    #[test]
    fn test_unit_conversion_advice() {
        assert!(generate_unit_conversion_advice("pressure_design").contains("145.038"));
        assert!(generate_unit_conversion_advice("thickness").contains("25.4"));
        assert!(generate_unit_conversion_advice("temperature").contains("9/5"));
        assert!(generate_unit_conversion_advice("stress_max").contains("145.038"));
        assert!(generate_unit_conversion_advice("unknown_param").contains("换算表"));
    }
}
