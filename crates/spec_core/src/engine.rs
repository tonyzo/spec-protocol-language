//! 合规引擎主流程
//!
//! 流程（参见 CONTEXT.md 数据流）：
//!   规则集 + 参数表
//!     → 适用性筛选(applicability)
//!     → 断言求值(assertion，内含量纲校验)
//!     → 生成 RuleResult / Violation
//!     → 汇总 ComplianceReport

use crate::assertion::{eval, EvalOutcome, SpecError};
use crate::model::{
    CheckStatus, ComplianceReport, DesignContext, ParameterTable, RuleResult, Severity,
    Violation,
};
use crate::rule::SpecRule;

/// 合规引擎。接收规则集 + 参数表，输出合规报告。
pub struct ComplianceEngine {
    rules: Vec<SpecRule>,
}

impl ComplianceEngine {
    pub fn new(rules: Vec<SpecRule>) -> Self {
        Self { rules }
    }

    /// 从 YAML 文本加载规则集。
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        let rules: Vec<SpecRule> = serde_yaml::from_str(yaml)?;
        Ok(Self::new(rules))
    }

    /// 执行全部规则校核。
    pub fn check(
        &self,
        table: &ParameterTable,
        context: DesignContext,
    ) -> ComplianceReport {
        log::info!("合规校核开始: {} 条规则", self.rules.len());
        if let Some(ref eid) = context.element_id {
            log::info!("  目标: {}", eid);
        }

        let results: Vec<RuleResult> = self
            .rules
            .iter()
            .map(|rule| check_rule(rule, table))
            .collect();

        let mut report = ComplianceReport {
            context,
            results,
            summary: Default::default(),
        };
        report.summarize();

        // 审计日志
        let s = &report.summary;
        log::info!(
            "合规校核完成: total={} pass={} fail={} ({} errors, {} warnings) na={} skip={}",
            s.total, s.passed, s.failed, s.errors, s.warnings, s.not_applicable, s.skipped
        );
        for r in &report.results {
            match r.status {
                CheckStatus::Pass => log::debug!("  [PASS] {} ({})", r.rule_id, r.clause),
                CheckStatus::Fail => log::warn!("  [FAIL] {} ({}): {}", r.rule_id, r.clause, r.message),
                CheckStatus::NotApplicable => log::debug!("  [N/A]  {} ({})", r.rule_id, r.clause),
                CheckStatus::Skipped => log::warn!("  [SKIP] {} ({}): {}", r.rule_id, r.clause, r.message),
            }
        }

        report
    }
}

/// 校核单条规则。
fn check_rule(rule: &SpecRule, table: &ParameterTable) -> RuleResult {
    // 1. 适用性筛选
    if let Some(app) = &rule.applicability {
        match eval(app, table) {
            Ok(EvalOutcome::Pass) => {} // 适用，继续
            Ok(EvalOutcome::Fail { .. }) => {
                return RuleResult {
                    rule_id: rule.id.clone(),
                    clause: rule.clause.clone(),
                    status: CheckStatus::NotApplicable,
                    severity: rule.severity,
                    message: "规则不适用".into(),
                    violations: vec![],
                };
            }
            Ok(EvalOutcome::Skipped) => {
                // 适用条件缺参数，保守地跳过
                return skipped(rule, "适用条件缺少参数");
            }
            Err(e) => {
                return skipped(rule, &format!("适用条件求值错误: {}", e));
            }
        }
    }

    // 2. 断言求值（内含量纲校验）
    match eval(&rule.assertion, table) {
        Ok(EvalOutcome::Pass) => RuleResult {
            rule_id: rule.id.clone(),
            clause: rule.clause.clone(),
            status: CheckStatus::Pass,
            severity: rule.severity,
            message: render_message(rule, None, None),
            violations: vec![],
        },
        Ok(EvalOutcome::Fail { actual, expected, unit }) => {
            let desc = rule.describe_assertion();
            let deviation = match (actual, expected) {
                (Some(a), Some(e)) => Some(a - e),
                _ => None,
            };
            let v = Violation {
                rule_id: rule.id.clone(),
                clause: rule.clause.clone(),
                assertion_desc: desc,
                actual,
                expected,
                unit,
                deviation,
            };
            RuleResult {
                rule_id: rule.id.clone(),
                clause: rule.clause.clone(),
                status: CheckStatus::Fail,
                severity: rule.severity,
                message: render_message(rule, actual, expected),
                violations: vec![v],
            }
        }
        Ok(EvalOutcome::Skipped) => skipped(rule, "断言缺少参数"),
        Err(e) => {
            // 量纲不一致等错误：升级为 error 级别结果
            RuleResult {
                rule_id: rule.id.clone(),
                clause: rule.clause.clone(),
                status: CheckStatus::Fail,
                severity: Severity::Error,
                message: format!("求值错误: {}", e),
                violations: vec![],
            }
        }
    }
}

fn skipped(rule: &SpecRule, reason: &str) -> RuleResult {
    RuleResult {
        rule_id: rule.id.clone(),
        clause: rule.clause.clone(),
        status: CheckStatus::Skipped,
        severity: rule.severity,
        message: reason.into(),
        violations: vec![],
    }
}

/// 渲染消息模板，替换 {actual} {expected} {rule_id}。
fn render_message(rule: &SpecRule, actual: Option<f64>, expected: Option<f64>) -> String {
    let tmpl = rule.message.clone().unwrap_or_default();
    let mut s = tmpl
        .replace("{rule_id}", &rule.id)
        .replace("{clause}", &rule.clause);
    s = s.replace(
        "{actual}",
        &actual.map(|v| v.to_string()).unwrap_or_default(),
    );
    s = s.replace(
        "{expected}",
        &expected.map(|v| v.to_string()).unwrap_or_default(),
    );
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Quantity;
    use std::collections::HashMap;

    fn make_table() -> ParameterTable {
        let mut parameters = HashMap::new();
        parameters.insert(
            "sigma".into(),
            Quantity { value: 120.0, unit: "MPa".into() },
        );
        let mut limits = HashMap::new();
        limits.insert(
            "sigma_allow".into(),
            Quantity { value: 163.0, unit: "MPa".into() },
        );
        let mut attrs = HashMap::new();
        attrs.insert("element_type".into(), "内压圆筒".into());
        ParameterTable { parameters, limits, attrs }
    }

    #[test]
    fn test_engine_pass() {
        let yaml = r#"
- id: GB150.3-5.3.sigma
  clause: "5.3"
  severity: error
  applicability:
    type: eq
    value: [{attr: element_type}, "内压圆筒"]
  assertion:
    type: le
    value: [{param: sigma}, {limit: sigma_allow}]
  message: "环向应力 {actual} <= 许用应力 {expected}"
"#;
        let engine = ComplianceEngine::from_yaml(yaml).unwrap();
        let report = engine.check(&make_table(), DesignContext::default());
        assert_eq!(report.summary.total, 1);
        assert_eq!(report.summary.passed, 1);
        assert_eq!(report.results[0].status, CheckStatus::Pass);
    }

    #[test]
    fn test_engine_not_applicable() {
        let yaml = r#"
- id: test
  clause: "5.3"
  severity: error
  applicability:
    type: eq
    value: [{attr: element_type}, "外压圆筒"]
  assertion:
    type: le
    value: [{param: sigma}, {limit: sigma_allow}]
"#;
        let engine = ComplianceEngine::from_yaml(yaml).unwrap();
        let report = engine.check(&make_table(), DesignContext::default());
        assert_eq!(report.results[0].status, CheckStatus::NotApplicable);
    }

    #[test]
    fn test_engine_fail() {
        let yaml = r#"
- id: test
  clause: "5.3"
  severity: error
  assertion:
    type: ge
    value: [{param: sigma}, {limit: sigma_allow}]
"#;
        let engine = ComplianceEngine::from_yaml(yaml).unwrap();
        let report = engine.check(&make_table(), DesignContext::default());
        assert_eq!(report.results[0].status, CheckStatus::Fail);
        assert!(!report.results[0].violations.is_empty());
        let v = &report.results[0].violations[0];
        assert_eq!(v.actual, Some(120.0));
        assert_eq!(v.expected, Some(163.0));
        assert_eq!(v.deviation, Some(-43.0));
    }
}
