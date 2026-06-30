//! 断言求值器 —— 执行比较与布尔运算
//!
//! 参见 ADR-001：这是比较/布尔运算，不是表达式求值引擎。
//! 参见 ADR-005：比较两个数值前校验 unit 一致性，不一致则 Err，不静默换算。

use crate::model::{ParameterTable, Quantity};
use crate::rule::{Assertion, Operand};

/// 求值错误（不可恢复）。
#[derive(Debug, thiserror::Error)]
pub enum SpecError {
    #[error("量纲不一致：{left} 与 {right}")]
    UnitMismatch { left: String, right: String },
    #[error("类型不匹配：数值与字符串比较")]
    TypeMismatch,
}

/// 解析后的值。
#[derive(Debug, Clone)]
enum Resolved {
    /// 数值 + 单位
    Qty(f64, String),
    /// 字符串
    Text(String),
}

impl Resolved {
    fn as_num(&self) -> Result<f64, ()> {
        match self {
            Resolved::Qty(v, _) => Ok(*v),
            Resolved::Text(_) => Err(()),
        }
    }
}

/// 求值结果。
#[derive(Debug, Clone)]
pub enum EvalOutcome {
    /// 断言成立
    Pass,
    /// 断言不成立，附带违规详情
    Fail {
        actual: Option<f64>,
        expected: Option<f64>,
        unit: Option<String>,
    },
    /// 缺少参数，无法判定
    Skipped,
}

/// 求值入口。
pub fn eval(a: &Assertion, table: &ParameterTable) -> Result<EvalOutcome, SpecError> {
    Ok(match a {
        Assertion::Ge(l, r) => compare(l, r, table, |a, b| a >= b)?,
        Assertion::Le(l, r) => compare(l, r, table, |a, b| a <= b)?,
        Assertion::Gt(l, r) => compare(l, r, table, |a, b| a > b)?,
        Assertion::Lt(l, r) => compare(l, r, table, |a, b| a < b)?,
        Assertion::Eq(l, r) => eq_op(l, r, table)?,
        Assertion::Ne(l, r) => match eq_op(l, r, table)? {
            EvalOutcome::Pass => EvalOutcome::Fail {
                actual: None,
                expected: None,
                unit: None,
            },
            EvalOutcome::Fail { .. } => EvalOutcome::Pass,
            EvalOutcome::Skipped => EvalOutcome::Skipped,
        },
        Assertion::And(parts) => {
            let mut skipped = false;
            for p in parts {
                match eval(p, table)? {
                    EvalOutcome::Pass => {}
                    EvalOutcome::Fail { actual, expected, unit } => {
                        return Ok(EvalOutcome::Fail { actual, expected, unit });
                    }
                    EvalOutcome::Skipped => skipped = true,
                }
            }
            if skipped {
                EvalOutcome::Skipped
            } else {
                EvalOutcome::Pass
            }
        }
        Assertion::Or(parts) => {
            let mut skipped = false;
            for p in parts {
                match eval(p, table)? {
                    EvalOutcome::Pass => return Ok(EvalOutcome::Pass),
                    EvalOutcome::Fail { .. } => {}
                    EvalOutcome::Skipped => skipped = true,
                }
            }
            if skipped {
                EvalOutcome::Skipped
            } else {
                EvalOutcome::Fail {
                    actual: None,
                    expected: None,
                    unit: None,
                }
            }
        }
        Assertion::Not(inner) => match eval(inner, table)? {
            EvalOutcome::Pass => EvalOutcome::Fail {
                actual: None,
                expected: None,
                unit: None,
            },
            EvalOutcome::Fail { .. } => EvalOutcome::Pass,
            EvalOutcome::Skipped => EvalOutcome::Skipped,
        },
        Assertion::Exists(op) => {
            if resolve(op, table).is_some() {
                EvalOutcome::Pass
            } else {
                EvalOutcome::Fail {
                    actual: None,
                    expected: None,
                    unit: None,
                }
            }
        }
        Assertion::When { cond, then } => match eval(cond, table)? {
            // 条件成立 → 要求 then 也成立
            EvalOutcome::Pass => eval(then, table)?,
            // 条件不成立 → 断言自动通过（不要求 then）
            EvalOutcome::Fail { .. } => EvalOutcome::Pass,
            EvalOutcome::Skipped => EvalOutcome::Skipped,
        },
    })
}

/// 数值比较。
fn compare(
    l: &Operand,
    r: &Operand,
    table: &ParameterTable,
    op: impl Fn(f64, f64) -> bool,
) -> Result<EvalOutcome, SpecError> {
    let (lv, rv) = match (resolve(l, table), resolve(r, table)) {
        (Some(lv), Some(rv)) => (lv, rv),
        _ => return Ok(EvalOutcome::Skipped),
    };
    let (ln, rn) = match (lv.as_num(), rv.as_num()) {
        (Ok(ln), Ok(rn)) => (ln, rn),
        _ => return Err(SpecError::TypeMismatch),
    };
    // 量纲一致性校验（ADR-005）
    // 数值字面量（Num）的 unit 为空字符串，视为"无单位约束"，不参与校验。
    // 仅当两侧 unit 均非空且不一致时才报错。
    if let (Resolved::Qty(_, lu), Resolved::Qty(_, ru)) = (&lv, &rv) {
        if !lu.is_empty() && !ru.is_empty() && lu != ru {
            return Err(SpecError::UnitMismatch {
                left: lu.clone(),
                right: ru.clone(),
            });
        }
    }
    let unit = match &lv {
        Resolved::Qty(_, u) => Some(u.clone()),
        _ => None,
    };
    if op(ln, rn) {
        Ok(EvalOutcome::Pass)
    } else {
        Ok(EvalOutcome::Fail {
            actual: Some(ln),
            expected: Some(rn),
            unit,
        })
    }
}

/// 相等比较（数值或字符串）。
fn eq_op(l: &Operand, r: &Operand, table: &ParameterTable) -> Result<EvalOutcome, SpecError> {
    let (lv, rv) = match (resolve(l, table), resolve(r, table)) {
        (Some(lv), Some(rv)) => (lv, rv),
        _ => return Ok(EvalOutcome::Skipped),
    };
    let equal = match (&lv, &rv) {
        (Resolved::Qty(lv, lu), Resolved::Qty(rv, ru)) => {
            // 同 compare()：空 unit 视为无约束，不校验
            if !lu.is_empty() && !ru.is_empty() && lu != ru {
                return Err(SpecError::UnitMismatch {
                    left: lu.clone(),
                    right: ru.clone(),
                });
            }
            (lv - rv).abs() < f64::EPSILON
        }
        (Resolved::Text(a), Resolved::Text(b)) => a == b,
        _ => return Err(SpecError::TypeMismatch),
    };
    Ok(if equal { EvalOutcome::Pass } else { EvalOutcome::Fail {
        actual: None,
        expected: None,
        unit: None,
    } })
}

/// 解析操作数到具体值。None 表示参数缺失。
fn resolve(op: &Operand, table: &ParameterTable) -> Option<Resolved> {
    match op {
        Operand::ParamRef { param } => table.parameters.get(param).map(|q| qty(q)),
        Operand::LimitRef { limit } => table.limits.get(limit).map(|q| qty(q)),
        Operand::AttrRef { attr } => table.attrs.get(attr).map(|s| Resolved::Text(s.clone())),
        Operand::Num(n) => Some(Resolved::Qty(*n, String::new())),
        Operand::Str(s) => Some(Resolved::Text(s.clone())),
    }
}

fn qty(q: &Quantity) -> Resolved {
    Resolved::Qty(q.value, q.unit.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Quantity;
    use std::collections::HashMap;

    fn table() -> ParameterTable {
        let mut parameters = HashMap::new();
        parameters.insert("delta".into(), Quantity { value: 16.0, unit: "mm".into() });
        parameters.insert("sigma".into(), Quantity { value: 120.0, unit: "MPa".into() });
        let mut limits = HashMap::new();
        limits.insert("delta_min".into(), Quantity { value: 12.0, unit: "mm".into() });
        limits.insert("sigma_allow".into(), Quantity { value: 163.0, unit: "MPa".into() });
        let mut attrs = HashMap::new();
        attrs.insert("element_type".into(), "内压圆筒".into());
        ParameterTable { parameters, limits, attrs }
    }

    #[test]
    fn test_pass() {
        let a = Assertion::Le(
            Operand::ParamRef { param: "sigma".into() },
            Operand::LimitRef { limit: "sigma_allow".into() },
        );
        assert!(matches!(eval(&a, &table()).unwrap(), EvalOutcome::Pass));
    }

    #[test]
    fn test_fail() {
        let a = Assertion::Ge(
            Operand::ParamRef { param: "delta".into() },
            Operand::Num(20.0),
        );
        assert!(matches!(eval(&a, &table()).unwrap(), EvalOutcome::Fail { .. }));
    }

    #[test]
    fn test_skipped() {
        let a = Assertion::Ge(
            Operand::ParamRef { param: "missing".into() },
            Operand::Num(1.0),
        );
        assert!(matches!(eval(&a, &table()).unwrap(), EvalOutcome::Skipped));
    }

    #[test]
    fn test_unit_mismatch() {
        let mut t = table();
        t.limits.insert("delta_min".into(), Quantity { value: 12.0, unit: "m".into() });
        let a = Assertion::Ge(
            Operand::ParamRef { param: "delta".into() },
            Operand::LimitRef { limit: "delta_min".into() },
        );
        assert!(matches!(eval(&a, &t), Err(SpecError::UnitMismatch { .. })));
    }

    #[test]
    fn test_when_condition_not_met() {
        // delta(16) > 38 为假 → When 自动 Pass
        let a = Assertion::When {
            cond: Box::new(Assertion::Gt(
                Operand::ParamRef { param: "delta".into() },
                Operand::Num(38.0),
            )),
            then: Box::new(Assertion::Exists(Operand::AttrRef { attr: "pwht".into() })),
        };
        assert!(matches!(eval(&a, &table()).unwrap(), EvalOutcome::Pass));
    }
}
