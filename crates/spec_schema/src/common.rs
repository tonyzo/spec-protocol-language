//! 共享基础类型 — 参数分类与规则严重度

use serde::{Deserialize, Serialize};

/// 参数类型枚举
/// - Parameter: 计算软件提供的计算值
/// - Limit: 标准规定的限值/阈值
/// - Attr: 分类属性(用于适用性筛选)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParamType {
    Parameter,
    Limit,
    Attr,
}

/// 规则严重度
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}
