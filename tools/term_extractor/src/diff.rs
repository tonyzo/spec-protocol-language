//! 差异比对模块
//!
//! 比较两个param_mapping.json的差异

use crate::models::ParamMapping;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// 差异类型
#[derive(Debug, Clone)]
pub enum DiffType {
    Added,      // 新增
    Removed,    // 删除
    Modified,   // 修改
}

/// 术语差异
#[derive(Debug, Clone)]
pub struct TermDiff {
    pub term_id: String,
    pub standard_term: String,
    pub diff_type: DiffType,
    pub details: Vec<String>,
}

/// 比对结果
#[derive(Debug, Clone)]
pub struct DiffResult {
    pub added: Vec<TermDiff>,
    pub removed: Vec<TermDiff>,
    pub modified: Vec<TermDiff>,
    pub unchanged: Vec<String>,
}

impl DiffResult {
    pub fn new() -> Self {
        DiffResult {
            added: Vec::new(),
            removed: Vec::new(),
            modified: Vec::new(),
            unchanged: Vec::new(),
        }
    }

    pub fn total_changes(&self) -> usize {
        self.added.len() + self.removed.len() + self.modified.len()
    }
}

/// 比对两个param_mapping.json
pub fn compare_mappings(
    old_path: &Path,
    new_path: &Path,
    output: Option<&Path>,
) -> anyhow::Result<DiffResult> {
    log::info!("开始比对术语映射...");
    log::info!("旧版本: {:?}", old_path);
    log::info!("新版本: {:?}", new_path);

    // 读取文件
    let old_content = fs::read_to_string(old_path)?;
    let new_content = fs::read_to_string(new_path)?;

    let old_mapping: ParamMapping = serde_json::from_str(&old_content)?;
    let new_mapping: ParamMapping = serde_json::from_str(&new_content)?;

    // 比对
    let result = compare_term_mappings(&old_mapping, &new_mapping)?;

    // 输出报告
    if let Some(output_path) = output {
        generate_diff_report(&result, output_path)?;
    }

    log::info!("比对完成! 发现 {} 处变更", result.total_changes());
    Ok(result)
}

/// 比对术语映射
fn compare_term_mappings(
    old_mapping: &ParamMapping,
    new_mapping: &ParamMapping,
) -> anyhow::Result<DiffResult> {
    let mut result = DiffResult::new();

    // 收集所有term_id
    let old_term_ids: HashSet<_> = old_mapping.terms.iter()
        .map(|t| t.term_id.as_str())
        .collect();
    let new_term_ids: HashSet<_> = new_mapping.terms.iter()
        .map(|t| t.term_id.as_str())
        .collect();

    // 查找新增的术语
    for term_id in &new_term_ids {
        if !old_term_ids.contains(term_id) {
            let new_term = new_mapping.terms.iter()
                .find(|t| t.term_id == *term_id)
                .unwrap();
            
            result.added.push(TermDiff {
                term_id: term_id.to_string(),
                standard_term: new_term.standard_term.clone(),
                diff_type: DiffType::Added,
                details: vec![
                    format!("新增术语: {}", new_term.standard_term),
                    format!("定义: {}", new_term.definition),
                ],
            });
        }
    }

    // 查找删除的术语
    for term_id in &old_term_ids {
        if !new_term_ids.contains(term_id) {
            let old_term = old_mapping.terms.iter()
                .find(|t| t.term_id == *term_id)
                .unwrap();
            
            result.removed.push(TermDiff {
                term_id: term_id.to_string(),
                standard_term: old_term.standard_term.clone(),
                diff_type: DiffType::Removed,
                details: vec![
                    format!("删除术语: {}", old_term.standard_term),
                    format!("原定义: {}", old_term.definition),
                ],
            });
        }
    }

    // 查找修改的术语
    for term_id in old_term_ids.intersection(&new_term_ids) {
        let old_term = old_mapping.terms.iter()
            .find(|t| t.term_id == *term_id)
            .unwrap();
        let new_term = new_mapping.terms.iter()
            .find(|t| t.term_id == *term_id)
            .unwrap();

        let mut details = Vec::new();

        // 比对各个字段
        if old_term.standard_term != new_term.standard_term {
            details.push(format!(
                "标准术语: '{}' -> '{}'",
                old_term.standard_term, new_term.standard_term
            ));
        }
        if old_term.definition != new_term.definition {
            details.push(format!(
                "定义: '{}' -> '{}'",
                old_term.definition, new_term.definition
            ));
        }
        if old_term.source_clause != new_term.source_clause {
            details.push(format!(
                "来源条款: '{}' -> '{}'",
                old_term.source_clause, new_term.source_clause
            ));
        }
        if old_term.spec_mapping.name != new_term.spec_mapping.name {
            details.push(format!(
                "SPEC参数名: '{}' -> '{}'",
                old_term.spec_mapping.name, new_term.spec_mapping.name
            ));
        }
        if old_term.spec_mapping.unit != new_term.spec_mapping.unit {
            details.push(format!(
                "单位: {:?} -> {:?}",
                old_term.spec_mapping.unit, new_term.spec_mapping.unit
            ));
        }

        if !details.is_empty() {
            result.modified.push(TermDiff {
                term_id: term_id.to_string(),
                standard_term: new_term.standard_term.clone(),
                diff_type: DiffType::Modified,
                details,
            });
        } else {
            result.unchanged.push(term_id.to_string());
        }
    }

    Ok(result)
}

/// 生成差异报告
fn generate_diff_report(result: &DiffResult, output: &Path) -> anyhow::Result<()> {
    let mut content = String::new();

    content.push_str("# 术语变更比对报告\n\n");
    content.push_str(&format!("**生成时间**: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));

    // 统计
    content.push_str("## 📊 变更统计\n\n");
    content.push_str(&format!("- **新增术语**: {} 个\n", result.added.len()));
    content.push_str(&format!("- **删除术语**: {} 个\n", result.removed.len()));
    content.push_str(&format!("- **修改术语**: {} 个\n", result.modified.len()));
    content.push_str(&format!("- **未变更**: {} 个\n", result.unchanged.len()));
    content.push_str(&format!("- **总变更数**: {} 处\n\n", result.total_changes()));

    // 新增术语
    if !result.added.is_empty() {
        content.push_str("## ➕ 新增术语\n\n");
        for diff in &result.added {
            content.push_str(&format!("### {}\n\n", diff.standard_term));
            content.push_str(&format!("- **term_id**: `{}`\n", diff.term_id));
            for detail in &diff.details {
                content.push_str(&format!("- {}\n", detail));
            }
            content.push_str("\n");
        }
    }

    // 删除术语
    if !result.removed.is_empty() {
        content.push_str("## ➖ 删除术语\n\n");
        for diff in &result.removed {
            content.push_str(&format!("### {}\n\n", diff.standard_term));
            content.push_str(&format!("- **term_id**: `{}`\n", diff.term_id));
            for detail in &diff.details {
                content.push_str(&format!("- {}\n", detail));
            }
            content.push_str("\n");
        }
    }

    // 修改术语
    if !result.modified.is_empty() {
        content.push_str("## ✏️ 修改术语\n\n");
        for diff in &result.modified {
            content.push_str(&format!("### {}\n\n", diff.standard_term));
            content.push_str(&format!("- **term_id**: `{}`\n", diff.term_id));
            content.push_str("**变更详情**:\n\n");
            for detail in &diff.details {
                content.push_str(&format!("- {}\n", detail));
            }
            content.push_str("\n");
        }
    }

    // 未变更
    if !result.unchanged.is_empty() {
        content.push_str("## ✅ 未变更术语\n\n");
        content.push_str("以下术语保持不变:\n\n");
        for term_id in &result.unchanged {
            content.push_str(&format!("- `{}`\n", term_id));
        }
        content.push_str("\n");
    }

    fs::write(output, content)?;
    log::info!("差异报告已生成: {:?}", output);

    Ok(())
}
