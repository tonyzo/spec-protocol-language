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
/// TODO: 待用户提供 RVM 协作格式样例后，补充字段映射与 schema。
/// 当前先输出结构化文本占位，确保数据流可验证。
pub struct RvmWriter;

impl ReportWriter for RvmWriter {
    fn write(&self, report: &ComplianceReport) -> Result<String, ReportError> {
        let mut out = String::new();
        out.push_str("# RVM Compliance Report (PDMS/E3D)\n");
        let ctx = &report.context;
        if let Some(eid) = &ctx.element_id {
            out.push_str(&format!("element: {}\n", eid));
        }
        if let Some(et) = &ctx.element_type {
            out.push_str(&format!("type: {}\n", et));
        }
        let s = &report.summary;
        out.push_str(&format!(
            "summary: total={} pass={} fail={} error={} warn={} na={} skip={}\n",
            s.total, s.passed, s.failed, s.errors, s.warnings, s.not_applicable, s.skipped
        ));
        out.push_str("--- violations ---\n");
        for r in &report.results {
            if r.status != CheckStatus::Fail {
                continue;
            }
            for v in &r.violations {
                out.push_str(&format!(
                    "[{}] {} (clause {}): {} | actual={} expected={} dev={}\n",
                    match r.severity {
                        Severity::Error => "ERROR",
                        Severity::Warning => "WARN",
                        Severity::Info => "INFO",
                    },
                    v.rule_id,
                    v.clause,
                    v.assertion_desc,
                    v.actual.map(|x| x.to_string()).unwrap_or_else(|| "n/a".into()),
                    v.expected.map(|x| x.to_string()).unwrap_or_else(|| "n/a".into()),
                    v.deviation.map(|x| x.to_string()).unwrap_or_else(|| "n/a".into()),
                ));
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
