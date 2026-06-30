//! 报告输出适配器（ADR-004）
//!
//! ReportWriter trait 抽象输出层：
//!   - JsonWriter：中性 JSON，用于测试/调试/前端消费。
//!   - RvmWriter：RVM 协作格式，对接 AVEVA PDMS/E3D。
//!     （具体 schema 待与 PDMS 侧对齐，先提供骨架。）

use crate::model::{CheckStatus, ComplianceReport, Severity};

#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    #[error("JSON 序列化失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("RVM 输出失败: {0}")]
    Rvm(String),
}

/// 报告输出器：将中性 ComplianceReport 转为目标格式。
pub trait ReportWriter {
    fn write(&self, report: &ComplianceReport) -> Result<String, ReportError>;
}

/// JSON 输出器。
pub struct JsonWriter;

impl ReportWriter for JsonWriter {
    fn write(&self, report: &ComplianceReport) -> Result<String, ReportError> {
        Ok(serde_json::to_string_pretty(report)?)
    }
}

/// RVM 协作格式输出器（对接 AVEVA PDMS/E3D）。
///
/// 输出结构化属性列表，模拟 PDMS 属性报告格式。
/// 字段映射参见 docs/rvm-schema.md。
///
/// TODO: 待与 PDMS 侧确认实际 RVM schema 后调整字段映射。
pub struct RvmWriter;

impl ReportWriter for RvmWriter {
    fn write(&self, report: &ComplianceReport) -> Result<String, ReportError> {
        let mut out = String::new();
        let ctx = &report.context;
        let s = &report.summary;

        // ── 头部元数据 ──
        out.push_str("# RVM Compliance Report\n");
        out.push_str(&format!("# Generated: {}\n", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")));
        if let Some(std) = &ctx.standard {
            out.push_str(&format!("# Standard: {}\n", std));
        }
        if let Some(eid) = &ctx.element_id {
            out.push_str(&format!("# Element: {}", eid));
            if let Some(et) = &ctx.element_type {
                out.push_str(&format!(" ({})", et));
            }
            out.push('\n');
        } else if let Some(et) = &ctx.element_type {
            out.push_str(&format!("# Element type: {}\n", et));
        }
        out.push_str(&format!(
            "# Summary: {} rules checked, {} passed, {} failed ({} errors, {} warnings)\n",
            s.total, s.passed, s.failed, s.errors, s.warnings
        ));
        out.push('\n');

        // ── 所有规则结果 ──
        out.push_str("# --- Rule Results ---\n\n");
        for r in &report.results {
            let status_tag = match r.status {
                CheckStatus::Pass => "PASS",
                CheckStatus::Fail => {
                    match r.severity {
                        Severity::Error => "FAIL(ERROR)",
                        Severity::Warning => "FAIL(WARN)",
                        Severity::Info => "FAIL(INFO)",
                    }
                }
                CheckStatus::NotApplicable => "N/A",
                CheckStatus::Skipped => "SKIP",
            };
            out.push_str(&format!("# [{}] {} (clause {})\n", status_tag, r.rule_id, r.clause));
            if !r.violations.is_empty() {
                for v in &r.violations {
                    out.push_str(&format!("#   {}", v.assertion_desc));
                    if let (Some(a), Some(e)) = (v.actual, v.expected) {
                        out.push_str(&format!(": {} vs {}", a, e));
                        if let Some(d) = v.deviation {
                            out.push_str(&format!(" (deviation: {})", d));
                        }
                    }
                    if let Some(u) = &v.unit {
                        out.push_str(&format!(" [{}]", u));
                    }
                    out.push('\n');
                }
            }
            out.push('\n');
        }

        // ── 违规详情（PDMS 属性格式）──
        if s.failed > 0 {
            out.push_str("# --- Violations (PDMS Attribute Format) ---\n\n");
            for r in &report.results {
                if r.status != CheckStatus::Fail {
                    continue;
                }
                for v in &r.violations {
                    out.push_str(&format!("RULE_ID\t{}\n", v.rule_id));
                    out.push_str(&format!("CLAUSE\t{}\n", v.clause));
                    out.push_str(&format!("ASSERTION\t{}\n", v.assertion_desc));
                    out.push_str(&format!("SEVERITY\t{}\n", match r.severity {
                        Severity::Error => "ERROR",
                        Severity::Warning => "WARNING",
                        Severity::Info => "INFO",
                    }));
                    out.push_str(&format!("ACTUAL\t{}\n", v.actual.map(|x| x.to_string()).unwrap_or_else(|| "n/a".into())));
                    out.push_str(&format!("EXPECTED\t{}\n", v.expected.map(|x| x.to_string()).unwrap_or_else(|| "n/a".into())));
                    if let Some(u) = &v.unit {
                        out.push_str(&format!("UNIT\t{}\n", u));
                    }
                    if let Some(d) = v.deviation {
                        out.push_str(&format!("DEVIATION\t{}\n", d));
                    }
                    out.push_str(&format!("MESSAGE\t{}\n", r.message));
                    out.push('\n');
                }
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_writer() {
        let report = ComplianceReport {
            context: Default::default(),
            results: vec![],
            summary: Default::default(),
        };
        let w = JsonWriter;
        let out = w.write(&report).unwrap();
        assert!(out.contains("\"summary\""));
    }

    #[test]
    fn test_rvm_writer() {
        let report = ComplianceReport {
            context: Default::default(),
            results: vec![],
            summary: Default::default(),
        };
        let w = RvmWriter;
        let out = w.write(&report).unwrap();
        assert!(out.contains("RVM Compliance Report"));
    }
}
