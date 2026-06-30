//! 术语提取模块
//!
//! 从Markdown文件中提取术语定义

use crate::models::{Confidence, ParamMapping, ParamType, SpecMapping, StandardInfo, TermEntry};
use regex::Regex;
use std::fs;
use std::path::Path;

/// 从Markdown文件提取术语
pub fn extract_terms(
    input: &Path,
    output: &Path,
    standard: &str,
    version: &str,
    parts: Vec<String>,
) -> anyhow::Result<()> {
    log::info!("开始从Markdown提取术语...");
    
    // 读取Markdown文件
    let content = fs::read_to_string(input)?;
    
    // 提取术语
    let terms = extract_from_markdown(&content)?;
    
    // 创建ParamMapping
    let mut mapping = ParamMapping::new(standard, version, parts);
    for term in terms {
        mapping.add_term(term);
    }
    
    // 写入JSON
    let json = serde_json::to_string_pretty(&mapping)?;
    fs::write(output.join("param_mapping.json"), json)?;
    
    log::info!("术语提取完成! 共提取 {} 个术语", mapping.terms.len());
    Ok(())
}

/// 从Markdown内容提取术语
fn extract_from_markdown(content: &str) -> anyhow::Result<Vec<TermEntry>> {
    let mut terms = Vec::new();
    
    // 查找术语定义表格
    // 格式: | 序号 | 标准术语 | 术语定义 | SPEC映射 | 数据类型 | 单位 | 来源条款 | 置信度 |
    let table_regex = Regex::new(r"\|\s*(\d+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|\s*([^|]+)\s*\|")?;
    
    for cap in table_regex.captures_iter(content) {
        let standard_term = cap[2].trim().to_string();
        let definition = cap[3].trim().to_string();
        let spec_name = cap[4].trim().to_string();
        let data_type = cap[5].trim().to_string();
        let unit = cap[6].trim().to_string();
        let source_clause = cap[7].trim().to_string();
        let confidence_str = cap[8].trim().to_string();
        
        // 跳过表头
        if standard_term == "标准术语" {
            continue;
        }
        
        // 推断参数类型
        let param_type = infer_param_type(&spec_name, &unit);
        
        // 创建SpecMapping
        let spec_mapping = SpecMapping {
            param_type: param_type.clone(),
            name: spec_name.clone(),
            unit: if unit == "-" || unit.is_empty() {
                None
            } else {
                Some(unit.clone())
            },
            data_type,
            allowed_values: None,
        };
        
        // 解析置信度
        let confidence = if confidence_str.contains("high") || confidence_str.contains("✅") {
            Confidence::High
        } else if confidence_str.contains("medium") || confidence_str.contains("⚠️") {
            Confidence::Medium
        } else {
            Confidence::Low
        };
        
        // 生成term_id
        let term_id = generate_term_id(&spec_name);
        
        let term = TermEntry {
            term_id,
            standard_term,
            definition,
            spec_mapping,
            source_clause,
            referenced_by: Vec::new(),
            related_terms: Vec::new(),
            confidence,
            note: None,
            synonyms: Vec::new(),
        };
        
        terms.push(term);
    }
    
    log::info!("从Markdown提取到 {} 个术语", terms.len());
    Ok(terms)
}

/// 推断参数类型
fn infer_param_type(spec_name: &str, unit: &str) -> ParamType {
    let name_lower = spec_name.to_lowercase();
    
    // 根据名称推断
    if name_lower.contains("type") || name_lower.contains("category") 
        || name_lower.contains("status") || name_lower.contains("done")
        || name_lower.contains("toxicity") || name_lower.contains("flammability")
    {
        return ParamType::Attr;
    }
    
    // 根据单位推断
    if unit == "-" || unit.is_empty() {
        return ParamType::Attr;
    }
    
    // 根据关键词推断
    if name_lower.contains("min") || name_lower.contains("max") 
        || name_lower.contains("allow") || name_lower.contains("required")
    {
        return ParamType::Limit;
    }
    
    // 默认是parameter
    ParamType::Parameter
}

/// 生成term_id
fn generate_term_id(spec_name: &str) -> String {
    // 将蛇形命名转换为点分
    let parts: Vec<&str> = spec_name.split('_').collect();
    
    if parts.len() >= 2 {
        format!("{}.{}", parts[0], parts[1..].join("_"))
    } else {
        spec_name.to_string()
    }
}
