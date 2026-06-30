//! 核心数据模型定义
//!
//! 定义术语提取工具使用的核心数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ParamType 已移至 spec_schema crate（与 spec_core 共享）
pub use spec_schema::ParamType;

/// 置信度级别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Confidence::High => write!(f, "high"),
            Confidence::Medium => write!(f, "medium"),
            Confidence::Low => write!(f, "low"),
        }
    }
}

/// SPEC参数映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecMapping {
    /// 参数类型
    #[serde(rename = "type")]
    pub param_type: ParamType,
    /// SPEC参数名(蛇形命名)
    pub name: String,
    /// 单位(attrs为null)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// 数据类型
    #[serde(default = "default_data_type")]
    pub data_type: String,
    /// 允许值(仅attr需要)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<String>>,
}

fn default_data_type() -> String {
    "f64".to_string()
}

/// 术语条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermEntry {
    /// 术语ID(英文点分,如pressure.design)
    pub term_id: String,
    /// 标准原文术语
    pub standard_term: String,
    /// 术语定义
    pub definition: String,
    /// SPEC参数映射
    pub spec_mapping: SpecMapping,
    /// 来源条款号
    pub source_clause: String,
    /// 哪些规则引用此术语
    #[serde(default)]
    pub referenced_by: Vec<String>,
    /// 相关术语
    #[serde(default)]
    pub related_terms: Vec<String>,
    /// 置信度
    #[serde(default = "default_confidence")]
    pub confidence: Confidence,
    /// 补充说明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// 同义词
    #[serde(default)]
    pub synonyms: Vec<String>,
}

fn default_confidence() -> Confidence {
    Confidence::High
}

/// 参数分组
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterGroups {
    /// 设计参数
    #[serde(default)]
    pub design: Vec<String>,
    /// 几何参数
    #[serde(default)]
    pub geometry: Vec<String>,
    /// 应力参数
    #[serde(default)]
    pub stress: Vec<String>,
    /// 限值参数
    #[serde(default)]
    pub limits: Vec<String>,
    /// 材料性能
    #[serde(default)]
    pub material: Vec<String>,
    /// 焊接与NDT
    #[serde(default)]
    pub welding_ndt: Vec<String>,
    /// 试验参数
    #[serde(default)]
    pub test: Vec<String>,
    /// 分类属性
    #[serde(default)]
    pub attrs: Vec<String>,
}

/// 跨标准引用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// 引用描述
    pub description: String,
    /// 涉及的术语
    pub terms: Vec<String>,
}

/// 校验规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// 规则描述
    pub rule: String,
    /// 校验逻辑(伪代码)
    pub check: String,
}

/// 标准信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardInfo {
    /// 标准名称
    pub name: String,
    /// 标准版本
    pub version: String,
    /// 标准分册
    pub parts: Vec<String>,
}

/// 参数映射表(完整结构)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamMapping {
    /// Schema版本
    #[serde(rename = "_schema_version")]
    pub schema_version: String,
    /// 描述
    #[serde(rename = "_description")]
    pub description: String,
    /// 标准号
    #[serde(rename = "_standard")]
    pub standard: String,
    /// 标准信息
    pub standard_info: StandardInfo,
    /// 术语列表
    pub terms: Vec<TermEntry>,
    /// 参数分组
    pub parameter_groups: ParameterGroups,
    /// 跨标准引用
    #[serde(default)]
    pub cross_references: HashMap<String, CrossReference>,
    /// 校验规则
    #[serde(default)]
    pub validation_rules: Vec<ValidationRule>,
}

impl ParamMapping {
    /// 创建空的参数映射表
    pub fn new(standard: &str, version: &str, parts: Vec<String>) -> Self {
        Self {
            schema_version: "1.0".to_string(),
            description: format!("{} 参数映射表", standard),
            standard: standard.to_string(),
            standard_info: StandardInfo {
                name: standard.to_string(),
                version: version.to_string(),
                parts,
            },
            terms: Vec::new(),
            parameter_groups: ParameterGroups {
                design: Vec::new(),
                geometry: Vec::new(),
                stress: Vec::new(),
                limits: Vec::new(),
                material: Vec::new(),
                welding_ndt: Vec::new(),
                test: Vec::new(),
                attrs: Vec::new(),
            },
            cross_references: HashMap::new(),
            validation_rules: Vec::new(),
        }
    }

    /// 添加术语条目
    pub fn add_term(&mut self, term: TermEntry) {
        // 自动添加到对应的参数分组
        let param_name = term.spec_mapping.name.clone();
        match term.spec_mapping.param_type {
            ParamType::Parameter => {
                if term.term_id.starts_with("pressure") || term.term_id.starts_with("temperature") {
                    self.parameter_groups.design.push(param_name);
                } else if term.term_id.starts_with("thickness")
                    || term.term_id.starts_with("geometry")
                {
                    self.parameter_groups.geometry.push(param_name);
                } else if term.term_id.starts_with("stress") {
                    self.parameter_groups.stress.push(param_name);
                }
            }
            ParamType::Limit => {
                self.parameter_groups.limits.push(param_name);
            }
            ParamType::Attr => {
                self.parameter_groups.attrs.push(param_name);
            }
        }
        self.terms.push(term);
    }

    /// 根据参数名查找术语
    pub fn find_term_by_param(&self, param_name: &str) -> Option<&TermEntry> {
        self.terms
            .iter()
            .find(|t| t.spec_mapping.name == param_name)
    }

    /// 检查参数是否存在
    pub fn has_param(&self, param_name: &str) -> bool {
        self.find_term_by_param(param_name).is_some()
    }

    /// 获取所有参数名
    pub fn all_param_names(&self) -> Vec<&str> {
        self.terms.iter().map(|t| t.spec_mapping.name.as_str()).collect()
    }
}
