//! 文档生成模块
//!
//! 从param_mapping.json生成术语定义表Markdown文档

use crate::models::{Confidence, ParamMapping, ParamType};
use std::fs;
use std::path::Path;

/// 生成术语定义表Markdown文档
pub fn generate_docs(mapping: &ParamMapping, output: &Path) -> anyhow::Result<()> {
    let mut content = String::new();
    
    // 标题
    content.push_str(&format!(
        "# {} 术语定义与概念映射表\n\n",
        mapping.standard_info.name
    ));
    
    content.push_str(&format!(
        "> 本文档将{}标准中的术语、概念与SPEC Protocol的参数/限值/属性建立映射关系。\n",
        mapping.standard_info.name
    ));
    content.push_str("> \n");
    content.push_str(&format!("> **数据来源**: {}\n", mapping.standard_info.parts.join(", ")));
    content.push_str("> \n");
    content.push_str("> **映射原则**: \n");
    content.push_str("> - 标准术语 → SPEC参数名(`parameters`/`limits`/`attrs`)\n");
    content.push_str("> - 明确哪些参数需计算软件提供\n");
    content.push_str("> - 标注术语出处条款号\n\n");
    content.push_str("---\n\n");
    
    // 按类别分组术语
    let pressure_terms: Vec<_> = mapping.terms.iter()
        .filter(|t| t.term_id.starts_with("pressure"))
        .collect();
    
    let temperature_terms: Vec<_> = mapping.terms.iter()
        .filter(|t| t.term_id.starts_with("temperature"))
        .collect();
    
    let thickness_terms: Vec<_> = mapping.terms.iter()
        .filter(|t| t.term_id.starts_with("thickness"))
        .collect();
    
    let stress_terms: Vec<_> = mapping.terms.iter()
        .filter(|t| t.term_id.starts_with("stress"))
        .collect();
    
    let geometry_terms: Vec<_> = mapping.terms.iter()
        .filter(|t| t.term_id.starts_with("geometry"))
        .collect();
    
    let material_terms: Vec<_> = mapping.terms.iter()
        .filter(|t| t.term_id.starts_with("material"))
        .collect();
    
    let attr_terms: Vec<_> = mapping.terms.iter()
        .filter(|t| matches!(t.spec_mapping.param_type, ParamType::Attr))
        .collect();
    
    // 1. 核心术语定义表
    content.push_str("## 1. 核心术语定义表\n\n");
    
    // 1.1 压力相关术语
    if !pressure_terms.is_empty() {
        content.push_str("### 1.1 压力相关术语\n\n");
        content.push_str(&generate_term_table(&pressure_terms));
    }
    
    // 1.2 温度相关术语
    if !temperature_terms.is_empty() {
        content.push_str("### 1.2 温度相关术语\n\n");
        content.push_str(&generate_term_table(&temperature_terms));
    }
    
    // 1.3 厚度相关术语
    if !thickness_terms.is_empty() {
        content.push_str("### 1.3 厚度相关术语 (关键!)\n\n");
        content.push_str(&generate_term_table(&thickness_terms));
    }
    
    // 1.4 应力与材料性能术语
    if !stress_terms.is_empty() {
        content.push_str("### 1.4 应力与材料性能术语\n\n");
        content.push_str(&generate_term_table(&stress_terms));
    }
    
    // 1.5 几何参数术语
    if !geometry_terms.is_empty() {
        content.push_str("### 1.5 几何参数术语\n\n");
        content.push_str(&generate_term_table(&geometry_terms));
    }
    
    // 1.6 材料性能术语
    if !material_terms.is_empty() {
        content.push_str("### 1.6 材料性能术语\n\n");
        content.push_str(&generate_term_table(&material_terms));
    }
    
    // 2. 分类属性术语
    if !attr_terms.is_empty() {
        content.push_str("## 2. 分类属性术语(attrs)\n\n");
        content.push_str(&generate_attr_table(&attr_terms));
    }
    
    // 3. 参数分组
    content.push_str("## 3. 参数分组速查\n\n");
    content.push_str("### 3.1 设计参数\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.design.join(", ")));
    
    content.push_str("### 3.2 几何参数\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.geometry.join(", ")));
    
    content.push_str("### 3.3 应力参数\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.stress.join(", ")));
    
    content.push_str("### 3.4 限值参数\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.limits.join(", ")));
    
    content.push_str("### 3.5 材料性能\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.material.join(", ")));
    
    content.push_str("### 3.6 焊接与NDT\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.welding_ndt.join(", ")));
    
    content.push_str("### 3.7 试验参数\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.test.join(", ")));
    
    content.push_str("### 3.8 分类属性\n");
    content.push_str(&format!("- {}\n\n", mapping.parameter_groups.attrs.join(", ")));
    
    // 4. 跨标准引用
    if !mapping.cross_references.is_empty() {
        content.push_str("## 4. 跨标准引用关系\n\n");
        for (key, cross_ref) in &mapping.cross_references {
            content.push_str(&format!("### {}\n", key));
            content.push_str(&format!("{}\n", cross_ref.description));
            content.push_str(&format!("**涉及术语**: {}\n\n", cross_ref.terms.join(", ")));
        }
    }
    
    // 5. 校验规则
    if !mapping.validation_rules.is_empty() {
        content.push_str("## 5. 校验规则\n\n");
        for rule in &mapping.validation_rules {
            content.push_str(&format!("- **{}**: {}\n", rule.rule, rule.check));
        }
    }
    
    // 页脚
    content.push_str("\n---\n\n");
    content.push_str("**文档版本**: 1.0  \n");
    content.push_str(&format!("**创建日期**: {}  \n", chrono::Local::now().format("%Y-%m-%d")));
    content.push_str(&format!("**数据来源**: {}  \n", mapping.standard_info.parts.join(", ")));
    content.push_str("**维护者**: SPEC Protocol 团队  \n");
    content.push_str("**状态**: 持续更新中\n");
    
    // 写入文件
    fs::write(output, content)?;
    log::info!("术语文档已生成: {:?}", output);
    
    Ok(())
}

/// 生成术语表格
fn generate_term_table(terms: &[&crate::models::TermEntry]) -> String {
    let mut table = String::new();
    
    table.push_str("| 序号 | 标准术语 | 术语定义 | SPEC映射 | 数据类型 | 单位 | 来源条款 | 置信度 |\n");
    table.push_str("|-----|---------|---------|---------|---------|-----|---------|--------|\n");
    
    for (idx, term) in terms.iter().enumerate() {
        let unit_str = term.spec_mapping.unit.clone().unwrap_or_else(|| "-".to_string());
        let data_type = &term.spec_mapping.data_type;
        let confidence = match term.confidence {
            Confidence::High => "✅ high",
            Confidence::Medium => "⚠️ medium",
            Confidence::Low => "❌ low",
        };
        
        table.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |\n",
            idx + 1,
            term.standard_term,
            term.definition,
            term.spec_mapping.name,
            data_type,
            unit_str,
            term.source_clause,
            confidence
        ));
    }
    
    table.push('\n');
    table
}

/// 生成属性表格
fn generate_attr_table(terms: &[&crate::models::TermEntry]) -> String {
    let mut table = String::new();
    
    table.push_str("| 属性名 | 可能值 | 说明 | 来源条款 |\n");
    table.push_str("|-------|-------|------|----------|\n");
    
    for term in terms {
        let allowed_values = term.spec_mapping.allowed_values.as_ref()
            .map(|v| v.join(", "))
            .unwrap_or_else(|| "-".to_string());
        
        table.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            term.spec_mapping.name,
            allowed_values,
            term.definition,
            term.source_clause
        ));
    }
    
    table.push('\n');
    table
}
