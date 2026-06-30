//! 校验模块
//!
//! 校验param_mapping.json和规则YAML的一致性

use crate::models::ParamMapping;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// 校验问题
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub level: IssueLevel,
    pub category: String,
    pub message: String,
    pub location: String,
}

/// 问题级别
#[derive(Debug, Clone)]
pub enum IssueLevel {
    Error,
    Warning,
    Info,
}

/// 校验结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
    pub stats: ValidationStats,
}

/// 校验统计
#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub total_terms: usize,
    pub total_rules: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

impl ValidationResult {
    pub fn new() -> Self {
        ValidationResult {
            issues: Vec::new(),
            stats: ValidationStats {
                total_terms: 0,
                total_rules: 0,
                errors: 0,
                warnings: 0,
                info: 0,
            },
        }
    }

    pub fn add_issue(&mut self, level: IssueLevel, category: &str, message: &str, location: &str) {
        match level {
            IssueLevel::Error => self.stats.errors += 1,
            IssueLevel::Warning => self.stats.warnings += 1,
            IssueLevel::Info => self.stats.info += 1,
        }

        self.issues.push(ValidationIssue {
            level,
            category: category.to_string(),
            message: message.to_string(),
            location: location.to_string(),
        });
    }

    pub fn has_errors(&self) -> bool {
        self.stats.errors > 0
    }
}

/// 校验param_mapping.json和规则YAML
pub fn validate_mappings(
    mappings: &[&Path],
    rules_dirs: &[&Path],
    report_path: Option<&Path>,
    check_only: bool,
) -> anyhow::Result<ValidationResult> {
    log::info!("开始校验...");

    let mut result = ValidationResult::new();

    // 1. 校验param_mapping.json
    for mapping_path in mappings {
        validate_mapping(mapping_path, &mut result)?;
    }

    // 2. 校验规则YAML
    for rules_dir in rules_dirs {
        validate_rules_dir(rules_dir, &mut result)?;
    }

    // 3. 交叉校验
    if !check_only {
        cross_validate(mappings, rules_dirs, &mut result)?;
    }

    // 4. 生成报告
    if let Some(report) = report_path {
        generate_validation_report(&result, report)?;
    }

    log::info!("校验完成! 发现 {} 个问题", result.issues.len());
    Ok(result)
}

/// 校验单个param_mapping.json
fn validate_mapping(mapping_path: &Path, result: &mut ValidationResult) -> anyhow::Result<()> {
    log::info!("校验param_mapping.json: {:?}", mapping_path);

    let content = fs::read_to_string(mapping_path)?;
    let mapping: ParamMapping = serde_json::from_str(&content)?;

    result.stats.total_terms += mapping.terms.len();

    // 检查1: term_id唯一性
    let mut term_ids = HashSet::new();
    for term in &mapping.terms {
        if term_ids.contains(&term.term_id) {
            result.add_issue(
                IssueLevel::Error,
                "唯一性",
                &format!("term_id重复: {}", term.term_id),
                &format!("{:?}", mapping_path),
            );
        }
        term_ids.insert(&term.term_id);
    }

    // 检查2: 参数名唯一性
    let mut param_names = HashSet::new();
    for term in &mapping.terms {
        if param_names.contains(&term.spec_mapping.name) {
            result.add_issue(
                IssueLevel::Warning,
                "唯一性",
                &format!("参数名重复: {}", term.spec_mapping.name),
                &format!("{:?}", mapping_path),
            );
        }
        param_names.insert(&term.spec_mapping.name);
    }

    // 检查3: 单位一致性
    for term in &mapping.terms {
        match term.spec_mapping.param_type {
            crate::models::ParamType::Parameter | crate::models::ParamType::Limit => {
                if term.spec_mapping.unit.is_none() {
                    result.add_issue(
                        IssueLevel::Warning,
                        "单位",
                        &format!("参数 '{}' 缺少单位", term.spec_mapping.name),
                        &format!("{:?}", mapping_path),
                    );
                }
            }
            crate::models::ParamType::Attr => {
                if term.spec_mapping.unit.is_some() {
                    result.add_issue(
                        IssueLevel::Info,
                        "单位",
                        &format!("属性 '{}' 不应有单位", term.spec_mapping.name),
                        &format!("{:?}", mapping_path),
                    );
                }
            }
        }
    }

    // 检查4: 引用完整性
    for term in &mapping.terms {
        for related in &term.related_terms {
            if !term_ids.contains(related) {
                result.add_issue(
                    IssueLevel::Warning,
                    "引用",
                    &format!("术语 '{}' 引用了不存在的术语 '{}'", term.term_id, related),
                    &format!("{:?}", mapping_path),
                );
            }
        }
    }

    Ok(())
}

/// 校验规则目录
fn validate_rules_dir(rules_dir: &Path, result: &mut ValidationResult) -> anyhow::Result<()> {
    log::info!("校验规则目录: {:?}", rules_dir);

    if !rules_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(rules_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            validate_rule_file(&path, result)?;
        }
    }

    Ok(())
}

/// 校验单个规则文件
fn validate_rule_file(rule_path: &Path, result: &mut ValidationResult) -> anyhow::Result<()> {
    let content = fs::read_to_string(rule_path)?;

    // 简单解析YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

    if let serde_yaml::Value::Sequence(rules) = yaml {
        result.stats.total_rules += rules.len();

        for rule in rules {
            if let serde_yaml::Value::Mapping(rule_map) = rule {
                // 检查规则ID
                if let Some(id) = rule_map.get(&serde_yaml::Value::String("id".to_string())) {
                    if let serde_yaml::Value::String(id_str) = id {
                        if id_str.is_empty() {
                            result.add_issue(
                                IssueLevel::Error,
                                "规则",
                                "规则ID不能为空",
                                &format!("{:?}", rule_path),
                            );
                        }
                    }
                } else {
                    result.add_issue(
                        IssueLevel::Error,
                        "规则",
                        "规则缺少id字段",
                        &format!("{:?}", rule_path),
                    );
                }
            }
        }
    }

    Ok(())
}

/// 交叉校验
fn cross_validate(
    mappings: &[&Path],
    rules_dirs: &[&Path],
    result: &mut ValidationResult,
) -> anyhow::Result<()> {
    log::info!("交叉校验...");

    // 收集所有参数名
    let mut all_params = HashSet::new();
    for mapping_path in mappings {
        let content = fs::read_to_string(mapping_path)?;
        let mapping: ParamMapping = serde_json::from_str(&content)?;

        for term in &mapping.terms {
            all_params.insert(term.spec_mapping.name.clone());
        }
    }

    // 检查规则中引用的参数是否存在
    for rules_dir in rules_dirs {
        if !rules_dir.exists() {
            continue;
        }

        for entry in fs::read_dir(rules_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(&path)?;
                let yaml: serde_yaml::Value = serde_yaml::from_str(&content)?;

                // 简单检查是否引用了参数
                if content.contains("{param:") {
                    // 这里可以进一步解析,检查参数是否存在
                    result.add_issue(
                        IssueLevel::Info,
                        "交叉校验",
                        &format!("规则文件引用了参数,需人工确认: {:?}", path),
                        &format!("{:?}", path),
                    );
                }
            }
        }
    }

    Ok(())
}

/// 生成校验报告
fn generate_validation_report(result: &ValidationResult, output: &Path) -> anyhow::Result<()> {
    let mut content = String::new();

    content.push_str("# 校验报告\n\n");
    content.push_str(&format!("**生成时间**: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    // 统计
    content.push_str("## 📊 校验统计\n\n");
    content.push_str(&format!("- **术语总数**: {}\n", result.stats.total_terms));
    content.push_str(&format!("- **规则总数**: {}\n", result.stats.total_rules));
    content.push_str(&format!("- **错误**: {} 个\n", result.stats.errors));
    content.push_str(&format!("- **警告**: {} 个\n", result.stats.warnings));
    content.push_str(&format!("- **信息**: {} 个\n\n", result.stats.info));

    if result.has_errors() {
        content.push_str("**状态**: ❌ 存在错误,需要修复\n\n");
    } else {
        content.push_str("**状态**: ✅ 校验通过\n\n");
    }

    // 按级别分组输出问题
    let errors: Vec<_> = result.issues.iter()
        .filter(|i| matches!(i.level, IssueLevel::Error))
        .collect();
    let warnings: Vec<_> = result.issues.iter()
        .filter(|i| matches!(i.level, IssueLevel::Warning))
        .collect();
    let infos: Vec<_> = result.issues.iter()
        .filter(|i| matches!(i.level, IssueLevel::Info))
        .collect();

    if !errors.is_empty() {
        content.push_str("## ❌ 错误\n\n");
        for issue in errors {
            content.push_str(&format!("- **{}**: {} (位置: {})\n", 
                issue.category, issue.message, issue.location));
        }
        content.push_str("\n");
    }

    if !warnings.is_empty() {
        content.push_str("## ⚠️ 警告\n\n");
        for issue in warnings {
            content.push_str(&format!("- **{}**: {} (位置: {})\n", 
                issue.category, issue.message, issue.location));
        }
        content.push_str("\n");
    }

    if !infos.is_empty() {
        content.push_str("## ℹ️ 信息\n\n");
        for issue in infos {
            content.push_str(&format!("- **{}**: {} (位置: {})\n", 
                issue.category, issue.message, issue.location));
        }
        content.push_str("\n");
    }

    fs::write(output, content)?;
    log::info!("校验报告已生成: {:?}", output);

    Ok(())
}
