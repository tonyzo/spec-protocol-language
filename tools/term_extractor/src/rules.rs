//! 规则生成模块
//!
//! 从param_mapping.json生成规则YAML骨架

use crate::models::ParamMapping;
use spec_schema::{Assertion, Operand, Severity, SpecRule};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// ── 规则模板系统 ───────────────────────────

/// 规则模板 — 可复用的规则生成模式
///
/// 替代字符串拼接，直接构造 SpecRule 结构体再序列化为 YAML，更类型安全。
pub enum RuleTemplate {
    /// 参数 vs 限值比较：`param >= limit`
    RangeCheck {
        id: String,
        clause: String,
        param: String,
        limit: String,
        op: CompareOp,
        element_type: String,
        message: String,
    },
    /// 属性存在性检查：`exists(attr)`
    ExistenceCheck {
        id: String,
        clause: String,
        attr: String,
        element_type: String,
        message: String,
    },
    /// 条件触发：`when param > threshold then exists(attr)`
    ConditionalCheck {
        id: String,
        clause: String,
        cond_param: String,
        threshold: f64,
        then_attr: String,
        element_type: String,
        message: String,
    },
}

/// 比较运算符
pub enum CompareOp {
    Ge, // >=
    Le, // <=
    Gt, // >
    Lt, // <
}

impl RuleTemplate {
    /// 实例化模板，生成 SpecRule
    pub fn instantiate(&self) -> SpecRule {
        match self {
            RuleTemplate::RangeCheck {
                id,
                clause,
                param,
                limit,
                op,
                element_type,
                message,
            } => SpecRule {
                id: id.clone(),
                clause: clause.clone(),
                severity: Severity::Error,
                applicability: Some(Assertion::Eq(
                    Operand::AttrRef { attr: "element_type".into() },
                    Operand::Str(element_type.clone()),
                )),
                assertion: match op {
                    CompareOp::Ge => Assertion::Ge(
                        Operand::ParamRef { param: param.clone() },
                        Operand::LimitRef { limit: limit.clone() },
                    ),
                    CompareOp::Le => Assertion::Le(
                        Operand::ParamRef { param: param.clone() },
                        Operand::LimitRef { limit: limit.clone() },
                    ),
                    CompareOp::Gt => Assertion::Gt(
                        Operand::ParamRef { param: param.clone() },
                        Operand::LimitRef { limit: limit.clone() },
                    ),
                    CompareOp::Lt => Assertion::Lt(
                        Operand::ParamRef { param: param.clone() },
                        Operand::LimitRef { limit: limit.clone() },
                    ),
                },
                message: Some(message.clone()),
            },
            RuleTemplate::ExistenceCheck {
                id,
                clause,
                attr,
                element_type,
                message,
            } => SpecRule {
                id: id.clone(),
                clause: clause.clone(),
                severity: Severity::Error,
                applicability: Some(Assertion::Eq(
                    Operand::AttrRef { attr: "element_type".into() },
                    Operand::Str(element_type.clone()),
                )),
                assertion: Assertion::Exists(Operand::AttrRef { attr: attr.clone() }),
                message: Some(message.clone()),
            },
            RuleTemplate::ConditionalCheck {
                id,
                clause,
                cond_param,
                threshold,
                then_attr,
                element_type,
                message,
            } => SpecRule {
                id: id.clone(),
                clause: clause.clone(),
                severity: Severity::Error,
                applicability: Some(Assertion::Eq(
                    Operand::AttrRef { attr: "element_type".into() },
                    Operand::Str(element_type.clone()),
                )),
                assertion: Assertion::When {
                    cond: Box::new(Assertion::Gt(
                        Operand::ParamRef { param: cond_param.clone() },
                        Operand::Num(*threshold),
                    )),
                    then: Box::new(Assertion::Exists(Operand::AttrRef { attr: then_attr.clone() })),
                },
                message: Some(message.clone()),
            },
        }
    }
}

/// 从模板列表生成规则 YAML 文件
pub fn generate_from_templates(
    templates: &[RuleTemplate],
    output: &Path,
) -> anyhow::Result<()> {
    let rules: Vec<SpecRule> = templates.iter().map(|t| t.instantiate()).collect();
    let yaml = serde_yaml::to_string(&rules)?;
    fs::write(output, yaml)?;
    log::info!("从模板生成 {} 条规则 → {:?}", rules.len(), output);
    Ok(())
}

/// 生成规则YAML骨架
pub fn generate_rules(mapping: &ParamMapping, output: &Path) -> anyhow::Result<()> {
    log::info!("开始生成规则YAML骨架...");
    
    // 创建输出目录
    fs::create_dir_all(output)?;
    
    // 按类别生成规则文件
    let rule_files = generate_rule_files(mapping)?;
    
    // 写入文件
    for (filename, content) in rule_files {
        let file_path = output.join(&filename);
        fs::write(&file_path, content)?;
        log::info!("生成规则文件: {:?}", file_path);
    }
    
    log::info!("规则YAML骨架生成完成!");
    Ok(())
}

/// 生成规则文件内容
fn generate_rule_files(mapping: &ParamMapping) -> anyhow::Result<Vec<(String, String)>> {
    let mut files = Vec::new();
    
    // 生成内压圆筒规则
    files.push(generate_internal_pressure_rules(mapping)?);
    
    // 生成外压圆筒规则
    files.push(generate_external_pressure_rules(mapping)?);
    
    // 生成封头规则
    files.push(generate_head_rules(mapping)?);
    
    // 生成法兰规则
    files.push(generate_flange_rules(mapping)?);
    
    // 生成焊接规则
    files.push(generate_welding_rules(mapping)?);
    
    // 生成NDT规则
    files.push(generate_ndt_rules(mapping)?);
    
    // 生成耐压试验规则
    files.push(generate_pressure_test_rules(mapping)?);
    
    Ok(files)
}

/// 生成内压圆筒规则
fn generate_internal_pressure_rules(mapping: &ParamMapping) -> anyhow::Result<(String, String)> {
    let mut content = String::new();
    
    content.push_str("# GB150.3-5.3 内压圆筒 —— 合规规则\n");
    content.push_str("#\n");
    content.push_str("# 规则库文件：候选规则，尚未绑定到任何设计类型。\n\n");
    
    // 规则1: 环向应力校核
    content.push_str("# ─── 规则 1：环向应力校核 (GB150.3 公式 5-5) ───\n");
    content.push_str("# σθ ≤ [σ]^t · φ\n");
    content.push_str("- id: GB150.3-5.3.sigma\n");
    content.push_str("  clause: \"5.3 / 公式 5-5\"\n");
    content.push_str("  severity: error\n");
    content.push_str("  applicability:\n");
    content.push_str("    type: eq\n");
    content.push_str("    value:\n");
    content.push_str("      - {attr: element_type}\n");
    content.push_str("      - \"内压圆筒\"\n");
    content.push_str("  assertion:\n");
    content.push_str("    type: le\n");
    content.push_str("    value:\n");
    if mapping.has_param("sigma") {
        content.push_str("      - {param: sigma}\n");
    }
    if mapping.has_param("sigma_allow") {
        content.push_str("      - {limit: sigma_allow}\n");
    }
    content.push_str("  message: \"环向应力 {actual} MPa 超过许用应力 {expected} MPa\"\n\n");
    
    // 规则2: 最小壁厚校核
    content.push_str("# ─── 规则 2：最小壁厚校核 (GB150.3 5.3) ───\n");
    content.push_str("# δ ≥ δ_min\n");
    content.push_str("- id: GB150.3-5.3.delta_min\n");
    content.push_str("  clause: \"5.3\"\n");
    content.push_str("  severity: error\n");
    content.push_str("  applicability:\n");
    content.push_str("    type: eq\n");
    content.push_str("    value:\n");
    content.push_str("      - {attr: element_type}\n");
    content.push_str("      - \"内压圆筒\"\n");
    content.push_str("  assertion:\n");
    content.push_str("    type: ge\n");
    content.push_str("    value:\n");
    if mapping.has_param("delta") {
        content.push_str("      - {param: delta}\n");
    }
    if mapping.has_param("delta_min") {
        content.push_str("      - {limit: delta_min}\n");
    }
    content.push_str("  message: \"壁厚 {actual} mm 小于最小允许壁厚 {expected} mm\"\n\n");
    
    // 规则3: 焊接系数范围
    content.push_str("# ─── 规则 3：焊接系数范围 (GB150.3 表 2) ───\n");
    content.push_str("# 0 < φ ≤ 1.0\n");
    content.push_str("- id: GB150.3-5.3.phi_range\n");
    content.push_str("  clause: \"表 2\"\n");
    content.push_str("  severity: error\n");
    content.push_str("  applicability:\n");
    content.push_str("    type: eq\n");
    content.push_str("    value:\n");
    content.push_str("      - {attr: element_type}\n");
    content.push_str("      - \"内压圆筒\"\n");
    content.push_str("  assertion:\n");
    content.push_str("    type: and\n");
    content.push_str("    value:\n");
    content.push_str("      - type: gt\n");
    content.push_str("        value:\n");
    if mapping.has_param("phi") {
        content.push_str("          - {param: phi}\n");
    }
    content.push_str("          - 0\n");
    content.push_str("      - type: le\n");
    content.push_str("        value:\n");
    if mapping.has_param("phi") {
        content.push_str("          - {param: phi}\n");
    }
    content.push_str("          - 1.0\n");
    content.push_str("  message: \"焊接接头系数 {actual} 超出允许范围 (0, 1.0]\"\n");
    
    Ok(("5.3-内压圆筒.yaml".to_string(), content))
}

/// 生成外压圆筒规则
fn generate_external_pressure_rules(mapping: &ParamMapping) -> anyhow::Result<(String, String)> {
    let mut content = String::new();
    
    content.push_str("# GB150.3-6 外压圆筒和球壳 —— 合规规则\n\n");
    
    // 规则1: 外压稳定性
    content.push_str("# ─── 规则 1：外压稳定性校核 (GB150.3 6.3.2) ───\n");
    content.push_str("# p_calc ≤ p_allow\n");
    content.push_str("- id: GB150.3-6.3.stability\n");
    content.push_str("  clause: \"6.3.2\"\n");
    content.push_str("  severity: error\n");
    content.push_str("  applicability:\n");
    content.push_str("    type: eq\n");
    content.push_str("    value:\n");
    content.push_str("      - {attr: element_type}\n");
    content.push_str("      - \"外压圆筒\"\n");
    content.push_str("  assertion:\n");
    content.push_str("    type: le\n");
    content.push_str("    value:\n");
    if mapping.has_param("calc_pressure") {
        content.push_str("      - {param: calc_pressure}\n");
    }
    if mapping.has_param("pressure_allow") {
        content.push_str("      - {limit: pressure_allow}\n");
    }
    content.push_str("  message: \"计算外压 {actual} MPa 超过许用外压 {expected} MPa\"\n");
    
    Ok(("6-外压圆筒和球壳.yaml".to_string(), content))
}

/// 生成封头规则
fn generate_head_rules(mapping: &ParamMapping) -> anyhow::Result<(String, String)> {
    let mut content = String::new();
    
    content.push_str("# GB150.3-7 封头 —— 合规规则\n\n");
    
    // 规则1: 椭圆封头应力
    content.push_str("# ─── 规则 1：椭圆封头应力校核 (GB150.3 7.3.2) ───\n");
    content.push_str("# σ ≤ [σ]^t · φ\n");
    content.push_str("- id: GB150.3-7.3.elliptical_stress\n");
    content.push_str("  clause: \"7.3.2\"\n");
    content.push_str("  severity: error\n");
    content.push_str("  applicability:\n");
    content.push_str("    type: eq\n");
    content.push_str("    value:\n");
    content.push_str("      - {attr: element_type}\n");
    content.push_str("      - \"椭圆形封头\"\n");
    content.push_str("  assertion:\n");
    content.push_str("    type: le\n");
    content.push_str("    value:\n");
    if mapping.has_param("sigma") {
        content.push_str("      - {param: sigma}\n");
    }
    if mapping.has_param("sigma_allow") {
        content.push_str("      - {limit: sigma_allow}\n");
    }
    content.push_str("  message: \"椭圆封头应力 {actual} MPa 超过许用应力 {expected} MPa\"\n");
    
    Ok(("7-封头.yaml".to_string(), content))
}

/// 生成法兰规则
fn generate_flange_rules(_mapping: &ParamMapping) -> anyhow::Result<(String, String)> {
    let mut content = String::new();
    
    content.push_str("# GB150.3-9 法兰 —— 合规规则\n\n");
    content.push_str("# (待补充)\n");
    
    Ok(("9-法兰.yaml".to_string(), content))
}

/// 生成焊接规则
fn generate_welding_rules(_mapping: &ParamMapping) -> anyhow::Result<(String, String)> {
    let mut content = String::new();
    
    content.push_str("# GB150.4-7 焊接 —— 合规规则\n\n");
    content.push_str("# (待补充)\n");
    
    Ok(("7-焊接.yaml".to_string(), content))
}

/// 生成NDT规则
fn generate_ndt_rules(_mapping: &ParamMapping) -> anyhow::Result<(String, String)> {
    let mut content = String::new();
    
    content.push_str("# GB150.4-10 无损检测 —— 合规规则\n\n");
    content.push_str("# (待补充)\n");
    
    Ok(("10-无损检测.yaml".to_string(), content))
}

/// 生成耐压试验规则
fn generate_pressure_test_rules(_mapping: &ParamMapping) -> anyhow::Result<(String, String)> {
    let mut content = String::new();
    
    content.push_str("# GB150.4-11 耐压试验 —— 合规规则\n\n");
    content.push_str("# (待补充)\n");
    
    Ok(("11-耐压试验.yaml".to_string(), content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_check_template_ge() {
        let template = RuleTemplate::RangeCheck {
            id: "test.ge".to_string(),
            clause: "5.3".to_string(),
            param: "delta".to_string(),
            limit: "delta_min".to_string(),
            op: CompareOp::Ge,
            element_type: "内压圆筒".to_string(),
            message: "壁厚不足".to_string(),
        };
        let rule = template.instantiate();
        assert_eq!(rule.id, "test.ge");
        assert_eq!(rule.severity, Severity::Error);
        assert!(rule.applicability.is_some());
        assert!(matches!(rule.assertion, Assertion::Ge(..)));
    }

    #[test]
    fn test_range_check_template_le() {
        let template = RuleTemplate::RangeCheck {
            id: "test.le".to_string(),
            clause: "5.3".to_string(),
            param: "sigma".to_string(),
            limit: "sigma_allow".to_string(),
            op: CompareOp::Le,
            element_type: "内压圆筒".to_string(),
            message: "应力超限".to_string(),
        };
        let rule = template.instantiate();
        assert!(matches!(rule.assertion, Assertion::Le(..)));
    }

    #[test]
    fn test_range_check_template_gt_lt() {
        let t_gt = RuleTemplate::RangeCheck {
            id: "test.gt".to_string(),
            clause: "5.3".to_string(),
            param: "p".to_string(),
            limit: "p_crit".to_string(),
            op: CompareOp::Gt,
            element_type: "内压圆筒".to_string(),
            message: "test".to_string(),
        };
        assert!(matches!(t_gt.instantiate().assertion, Assertion::Gt(..)));

        let t_lt = RuleTemplate::RangeCheck {
            id: "test.lt".to_string(),
            clause: "5.3".to_string(),
            param: "p".to_string(),
            limit: "p_crit".to_string(),
            op: CompareOp::Lt,
            element_type: "内压圆筒".to_string(),
            message: "test".to_string(),
        };
        assert!(matches!(t_lt.instantiate().assertion, Assertion::Lt(..)));
    }

    #[test]
    fn test_existence_check_template() {
        let template = RuleTemplate::ExistenceCheck {
            id: "test.exists".to_string(),
            clause: "7.3".to_string(),
            attr: "pwht_done".to_string(),
            element_type: "内压圆筒".to_string(),
            message: "需要焊后热处理".to_string(),
        };
        let rule = template.instantiate();
        assert_eq!(rule.id, "test.exists");
        assert!(rule.applicability.is_some());
        assert!(matches!(rule.assertion, Assertion::Exists(..)));
    }

    #[test]
    fn test_conditional_check_template() {
        let template = RuleTemplate::ConditionalCheck {
            id: "test.conditional".to_string(),
            clause: "7.3".to_string(),
            cond_param: "delta".to_string(),
            threshold: 38.0,
            then_attr: "pwht_done".to_string(),
            element_type: "内压圆筒".to_string(),
            message: "壁厚超过38mm需要PWHT".to_string(),
        };
        let rule = template.instantiate();
        assert!(matches!(rule.assertion, Assertion::When { .. }));
        assert!(rule.message.is_some());
    }
}
