//! 规则定义 —— SpecRule + Assertion 枚举
//!
//! 参见 CONTEXT.md「工艺合规判据形态」。
//! Assertion 是结构化的布尔判定树，支持 4 种判据：
//!   1. 存在性检查 (Exists)
//!   2. 参数范围比对 (Ge/Le/Gt/Lt/Eq/Ne)
//!   3. 组合条件逻辑 (And/Or/Not)
//!   4. 条件触发 (When)
//! 注意：这不是表达式求值引擎（ADR-001），仅做比较与布尔运算。

use crate::model::Severity;

/// 操作数：断言中参与比较的值。
/// 通过 YAML 反序列化，用 untagged 区分。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Operand {
    /// 引用数值参数：{param: delta}
    ParamRef { param: String },
    /// 引用限值：{limit: delta_min}
    LimitRef { limit: String },
    /// 引用分类属性（字符串）：{attr: element_type}
    AttrRef { attr: String },
    /// 数值字面量：38
    Num(f64),
    /// 字符串字面量："内压圆筒"
    Str(String),
}

/// 断言：结构化布尔判定树。
/// 既用于 `assertion`（合规判据），也用于 `applicability`（适用条件）。
/// 使用 adjacently tagged 格式（serde_yaml 0.9 兼容）：
///   type: eq
///   value: [...]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum Assertion {
    // —— 比较运算（数值或字符串相等性）——
    Ge(Operand, Operand),
    Le(Operand, Operand),
    Gt(Operand, Operand),
    Lt(Operand, Operand),
    Eq(Operand, Operand),
    Ne(Operand, Operand),

    // —— 布尔组合 ——
    And(Vec<Assertion>),
    Or(Vec<Assertion>),
    Not(Box<Assertion>),

    // —— 存在性检查 ——
    /// Exists(Operand)：检查参数/限值/属性是否存在（key 存在）。
    Exists(Operand),

    // —— 条件触发：when cond then assert ——
    When {
        cond: Box<Assertion>,
        then: Box<Assertion>,
    },
}

/// 规范规则：GB150 中一条可机器判定的合规要求。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpecRule {
    /// 规则 ID，如 "GB150.3-5.3.sigma"
    pub id: String,
    /// GB150 条款号，如 "5.3"
    pub clause: String,
    pub severity: Severity,
    /// 适用条件：命中则规则生效，否则 NotApplicable。
    /// 缺省时视为恒真（始终适用）。
    #[serde(default)]
    pub applicability: Option<Assertion>,
    /// 合规判据。
    pub assertion: Assertion,
    /// 人类可读消息模板，支持占位符 {actual} {expected} {rule_id}。
    #[serde(default)]
    pub message: Option<String>,
}

impl SpecRule {
    /// 生成断言的人类可读描述（用于报告溯源）。
    pub fn describe_assertion(&self) -> String {
        describe(&self.assertion)
    }
}

/// 递归生成断言的字符串描述。
fn describe(a: &Assertion) -> String {
    use Assertion::*;
    match a {
        Ge(l, r) => format!("{} >= {}", op_str(l), op_str(r)),
        Le(l, r) => format!("{} <= {}", op_str(l), op_str(r)),
        Gt(l, r) => format!("{} > {}", op_str(l), op_str(r)),
        Lt(l, r) => format!("{} < {}", op_str(l), op_str(r)),
        Eq(l, r) => format!("{} == {}", op_str(l), op_str(r)),
        Ne(l, r) => format!("{} != {}", op_str(l), op_str(r)),
        And(v) => v.iter().map(describe).collect::<Vec<_>>().join(" AND "),
        Or(v) => v.iter().map(describe).collect::<Vec<_>>().join(" OR "),
        Not(inner) => format!("NOT({})", describe(inner)),
        Exists(o) => format!("exists({})", op_str(o)),
        When { cond, then } => format!("when({}) then({})", describe(cond), describe(then)),
    }
}

fn op_str(o: &Operand) -> String {
    use Operand::*;
    match o {
        ParamRef { param } => format!("param.{}", param),
        LimitRef { limit } => format!("limit.{}", limit),
        AttrRef { attr } => format!("attr.{}", attr),
        Num(n) => n.to_string(),
        Str(s) => format!("\"{}\"", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_simple_rule() {
        let yaml = r#"
id: GB150.3-5.3.sigma
clause: "5.3"
severity: error
applicability:
  type: eq
  value: [{attr: element_type}, "内压圆筒"]
assertion:
  type: le
  value: [{param: sigma}, {limit: sigma_allow}]
message: "环向应力 {actual} 超过许用应力 {expected}"
"#;
        let rule: SpecRule = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(rule.clause, "5.3");
        assert_eq!(rule.severity, Severity::Error);
        assert!(rule.applicability.is_some());
    }

    #[test]
    fn test_deserialize_when_rule() {
        let yaml = r#"
id: GB150.4-7.PWHT
clause: "7.3"
severity: error
assertion:
  type: when
  value:
    cond:
      type: gt
      value: [{param: delta}, 38]
    then:
      type: exists
      value: {attr: pwht_done}
"#;
        let rule: SpecRule = serde_yaml::from_str(yaml).unwrap();
        assert!(matches!(rule.assertion, Assertion::When { .. }));
    }
}
