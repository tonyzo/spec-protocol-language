//! 审查点发现引擎 (ADR-006)
//!
//! 管理审查点生命周期：发现 → 确认 → 沉淀 → 加载。
//!
//! ```text
//! RuleLibrary ──discover()──► CandidateReviewPoint[]
//!                                  │
//!                                  │ 人工确认
//!                                  ▼
//!                            SpecRule[] (已确认)
//!                                  │
//!                                  │ sediment()
//!                                  ▼
//!                      rules/confirmed/{type}/{std}-{clause}-{year}.yaml
//!                      + registry.json 更新
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::assertion::{eval, EvalOutcome, SpecError};
use crate::model::{
    CandidateMatchStatus, CandidateReviewPoint, ParameterTable, RegistryEntry,
    ReviewPointRegistry,
};
use crate::registry::{RegistryError, RuleLibrary};
use crate::rule::SpecRule;

#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("注册表错误: {0}")]
    Registry(#[from] RegistryError),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML 序列化错误: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("JSON 序列化错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("设计类型 '{0}' 无已确认审查点集合")]
    NoConfirmedSet(String),
    #[error("审查点文件不存在: {0}")]
    FileNotFound(String),
    #[error("规则 ID 无法解析标准号: {0}")]
    InvalidRuleId(String),
}

/// 发现引擎：管理审查点生命周期。
pub struct DiscoveryEngine {
    /// 规则库（所有候选规则）
    library: RuleLibrary,
    /// 审查点注册表
    registry: ReviewPointRegistry,
    /// 注册表文件路径
    registry_path: PathBuf,
    /// 已确认审查点集合的存储根目录
    confirmed_dir: PathBuf,
}

impl DiscoveryEngine {
    /// 创建发现引擎。
    ///
    /// - `library`: 规则库（从 `rules/library/` 加载）
    /// - `registry`: 审查点注册表
    /// - `registry_path`: 注册表文件路径（如 `rules/registry.json`）
    /// - `confirmed_dir`: 审查点集合根目录（如 `rules/confirmed/`）
    pub fn new(
        library: RuleLibrary,
        registry: ReviewPointRegistry,
        registry_path: PathBuf,
        confirmed_dir: PathBuf,
    ) -> Self {
        Self {
            library,
            registry,
            registry_path,
            confirmed_dir,
        }
    }

    /// 从标准目录结构创建发现引擎。
    ///
    /// 期望目录结构:
    /// ```text
    /// rules/
    /// ├── library/      # 规则库
    /// ├── confirmed/    # 审查点集合
    /// └── registry.json
    /// ```
    pub fn from_rules_dir(rules_dir: &Path) -> Result<Self, DiscoveryError> {
        let library_dir = rules_dir.join("library");
        let confirmed_dir = rules_dir.join("confirmed");
        let registry_path = rules_dir.join("registry.json");

        let library = RuleLibrary::from_dir(&library_dir)?;
        let registry = ReviewPointRegistry::load(&registry_path)?;

        Ok(Self::new(library, registry, registry_path, confirmed_dir))
    }

    /// 获取规则库引用。
    pub fn library(&self) -> &RuleLibrary {
        &self.library
    }

    /// 获取注册表引用。
    pub fn registry(&self) -> &ReviewPointRegistry {
        &self.registry
    }

    // ─── 检查与加载已确认审查点 ───

    /// 检查设计类型是否已有审查点集合。
    pub fn has_confirmed_set(&self, design_type: &str) -> bool {
        self.registry.has_design_type(design_type)
    }

    /// 加载设计类型的全部已确认审查点。
    ///
    /// 读取注册表中该设计类型的所有条目，逐个加载对应的 YAML 文件。
    pub fn load_confirmed(&self, design_type: &str) -> Result<Vec<SpecRule>, DiscoveryError> {
        let entries = self
            .registry
            .get_entries(design_type)
            .ok_or_else(|| DiscoveryError::NoConfirmedSet(design_type.into()))?;

        let mut all_rules = Vec::new();
        for entry in entries {
            let file_path = self
                .confirmed_dir
                .join(design_type)
                .join(&entry.file_path);

            if !file_path.exists() {
                return Err(DiscoveryError::FileNotFound(file_path.display().to_string()));
            }

            let yaml_text = fs::read_to_string(&file_path)?;
            let rules: Vec<SpecRule> = serde_yaml::from_str(&yaml_text)?;
            all_rules.extend(rules);
        }

        Ok(all_rules)
    }

    // ─── 发现候选审查点 ───

    /// 从规则库中发现候选审查点。
    ///
    /// 对每条规则评估其 `applicability`，根据匹配状态分类：
    /// - `Applicable`: applicability 命中 → 强候选
    /// - `Uncertain`: applicability 缺参数 → 弱候选（需人工判断）
    /// - `NoApplicability`: 无适用条件 → 默认候选
    ///
    /// 注意：applicability 不命中的规则**不会**出现在候选列表中。
    pub fn discover(&self, table: &ParameterTable) -> Vec<CandidateReviewPoint> {
        let mut candidates = Vec::new();

        for rule in self.library.rules() {
            match &rule.applicability {
                None => {
                    // 无适用条件 → 默认候选
                    candidates.push(CandidateReviewPoint {
                        rule: rule.clone(),
                        match_status: CandidateMatchStatus::NoApplicability,
                        reason: "无适用条件定义，默认候选".into(),
                    });
                }
                Some(applicability) => {
                    match eval(applicability, table) {
                        Ok(EvalOutcome::Pass) => {
                            candidates.push(CandidateReviewPoint {
                                rule: rule.clone(),
                                match_status: CandidateMatchStatus::Applicable,
                                reason: "applicability 命中".into(),
                            });
                        }
                        Ok(EvalOutcome::Fail { .. }) => {
                            // 不适用，不加入候选
                        }
                        Ok(EvalOutcome::Skipped) => {
                            candidates.push(CandidateReviewPoint {
                                rule: rule.clone(),
                                match_status: CandidateMatchStatus::Uncertain,
                                reason: "applicability 缺少参数，需人工判断".into(),
                            });
                        }
                        Err(e) => {
                            // 求值错误（如量纲不一致）→ 弱候选，附带错误信息
                            candidates.push(CandidateReviewPoint {
                                rule: rule.clone(),
                                match_status: CandidateMatchStatus::Uncertain,
                                reason: format!("applicability 求值错误: {}", e),
                            });
                        }
                    }
                }
            }
        }

        candidates
    }

    // ─── 沉淀审查点 ───

    /// 将已确认的审查点沉淀为 YAML 文件并更新注册表。
    ///
    /// - `design_type`: 设计类型，如 "内压圆筒"
    /// - `confirmed_rules`: 已确认的规则列表
    /// - `standard_year`: 标准年份，如 "2024"
    ///
    /// 规则按标准号分组，每个标准保存为独立文件：
    /// `rules/confirmed/{design_type}/{standard}-{clause}-{year}.yaml`
    pub fn sediment(
        &mut self,
        design_type: &str,
        confirmed_rules: &[SpecRule],
        standard_year: &str,
    ) -> Result<Vec<PathBuf>, DiscoveryError> {
        // 按标准号分组
        let groups = group_rules_by_standard(confirmed_rules)?;

        // 确保设计类型目录存在
        let type_dir = self.confirmed_dir.join(design_type);
        fs::create_dir_all(&type_dir)?;

        let mut saved_files = Vec::new();
        let mut new_entries = Vec::new();

        for (standard, rules) in &groups {
            // 提取条款号（取第一条规则的 clause，去掉 "/" 后面的部分）
            let clause = rules
                .first()
                .map(|r| r.clause.split('/').next().unwrap_or("unknown").trim().to_string())
                .unwrap_or_else(|| "unknown".into());

            let filename = format!("{}-{}-{}.yaml", standard, clause, standard_year);
            let file_path = type_dir.join(&filename);

            // 序列化为 YAML
            let yaml_text = serde_yaml::to_string(rules)?;
            fs::write(&file_path, yaml_text)?;

            let entry = RegistryEntry {
                standard: standard.clone(),
                clause: clause.clone(),
                file_path: filename,
                rule_count: rules.len(),
                confirmed_at: current_iso_timestamp(),
            };

            new_entries.push(entry);
            saved_files.push(file_path);
        }

        // 更新注册表：替换该设计类型的全部条目
        self.registry.design_types.insert(design_type.to_string(), new_entries);

        // 保存注册表
        self.registry.save(&self.registry_path)?;

        Ok(saved_files)
    }
}

/// 按标准号分组规则。
/// 标准号从规则 ID 的前缀提取，如 "GB150.3-5.3.sigma" → "GB150.3"。
fn group_rules_by_standard(
    rules: &[SpecRule],
) -> Result<HashMap<String, Vec<SpecRule>>, DiscoveryError> {
    let mut groups: HashMap<String, Vec<SpecRule>> = HashMap::new();

    for rule in rules {
        let standard = extract_standard(&rule.id)
            .ok_or_else(|| DiscoveryError::InvalidRuleId(rule.id.clone()))?;

        groups
            .entry(standard)
            .or_default()
            .push(rule.clone());
    }

    Ok(groups)
}

/// 从规则 ID 提取标准号。
/// "GB150.3-5.3.sigma" → "GB150.3"
/// "GB150.4-7.PWHT_carbon" → "GB150.4"
/// "NB47012-3.1.rt" → "NB47012"
fn extract_standard(rule_id: &str) -> Option<String> {
    // 规则 ID 格式: {标准号}-{条款号}.{描述}
    // 标准号: 第一段 "-" 之前（含标准号中的 "."）
    // 但需要区分标准号中的 "." 和条款号
    // 实际规则: ID 中第一个 "-" 分隔标准号和条款号
    // 但标准号可能不含 "-"，如 "GB150.3" 而非 "GB-150.3"
    // 策略: 取第一个 "-" 之前的部分作为标准号候选，
    //       但如果候选中包含 "."，说明标准号已完整（如 "GB150.3"）
    //       如果不含 "."，可能标准号本身就是第一个 "-" 前（如 "NB47012"）
    let first_dash = rule_id.find('-')?;
    let candidate = &rule_id[..first_dash];
    // 如果候选包含 "." 或字母+数字，视为标准号
    if candidate.contains('.') || candidate.chars().any(|c| c.is_alphabetic()) {
        Some(candidate.to_string())
    } else {
        Some(candidate.to_string())
    }
}

/// 生成当前时间的 ISO 8601 字符串。
/// 注意：不依赖 chrono，用 SystemTime 手动格式化（精度到秒）。
fn current_iso_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // 简化版时间戳：Unix epoch 秒数（不格式化为 ISO 8601 以避免引入 chrono 依赖）
    format!("epoch:{}", secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Quantity;
    use std::collections::HashMap;

    fn make_table() -> ParameterTable {
        let mut parameters = HashMap::new();
        parameters.insert(
            "delta".into(),
            Quantity { value: 16.0, unit: "mm".into() },
        );
        parameters.insert(
            "sigma".into(),
            Quantity { value: 120.0, unit: "MPa".into() },
        );
        let mut limits = HashMap::new();
        limits.insert(
            "sigma_allow".into(),
            Quantity { value: 163.0, unit: "MPa".into() },
        );
        let mut attrs = HashMap::new();
        attrs.insert("element_type".into(), "内压圆筒".into());
        ParameterTable { parameters, limits, attrs }
    }

    fn make_library() -> RuleLibrary {
        let yaml = r#"
- id: GB150.3-5.3.sigma
  clause: "5.3 / 公式 5-5"
  severity: error
  applicability:
    type: eq
    value: [{attr: element_type}, "内压圆筒"]
  assertion:
    type: le
    value: [{param: sigma}, {limit: sigma_allow}]

- id: GB150.3-5.3.delta_min
  clause: "5.3 / 公式 5-1"
  severity: error
  applicability:
    type: eq
    value: [{attr: element_type}, "内压圆筒"]
  assertion:
    type: ge
    value: [{param: delta}, {limit: delta_min}]

- id: GB150.4-9.NDT
  clause: "9.2"
  severity: warning
  applicability:
    type: eq
    value: [{attr: element_type}, "外压圆筒"]
  assertion:
    type: ge
    value: [{param: rt_ratio}, 20]

- id: general.1
  clause: "1.1"
  severity: info
  assertion:
    type: ge
    value: [{param: sigma}, 0]
"#;
        RuleLibrary::from_yaml(yaml).unwrap()
    }

    #[test]
    fn test_extract_standard() {
        assert_eq!(extract_standard("GB150.3-5.3.sigma"), Some("GB150.3".into()));
        assert_eq!(extract_standard("GB150.4-7.PWHT_carbon"), Some("GB150.4".into()));
        assert_eq!(extract_standard("NB47012-3.1.rt"), Some("NB47012".into()));
    }

    #[test]
    fn test_discover_filters_applicability() {
        let lib = make_library();
        let registry = ReviewPointRegistry::default();
        let engine = DiscoveryEngine::new(
            lib,
            registry,
            PathBuf::from("/tmp/registry.json"),
            PathBuf::from("/tmp/confirmed"),
        );

        let candidates = engine.discover(&make_table());

        // 4 条规则中:
        // - GB150.3-5.3.sigma: applicability 命中 → Applicable
        // - GB150.3-5.3.delta_min: applicability 命中但缺 delta_min → Skipped → Uncertain
        //   (wait, delta_min is in limits, not provided in make_table, so Skipped)
        //   Actually, delta_min is not in the table, so applicability eval should still Pass
        //   because applicability only checks element_type, not delta_min.
        //   So GB150.3-5.3.delta_min → Applicable
        // - GB150.4-9.NDT: applicability 不命中（外压圆筒）→ 排除
        // - general.1: 无 applicability → NoApplicability
        assert_eq!(candidates.len(), 3);

        let applicable: Vec<_> = candidates
            .iter()
            .filter(|c| c.match_status == CandidateMatchStatus::Applicable)
            .collect();
        assert_eq!(applicable.len(), 2);

        let no_app: Vec<_> = candidates
            .iter()
            .filter(|c| c.match_status == CandidateMatchStatus::NoApplicability)
            .collect();
        assert_eq!(no_app.len(), 1);
        assert_eq!(no_app[0].rule.id, "general.1");
    }

    #[test]
    fn test_discover_excludes_not_applicable() {
        let lib = make_library();
        let registry = ReviewPointRegistry::default();
        let engine = DiscoveryEngine::new(
            lib,
            registry,
            PathBuf::from("/tmp/registry.json"),
            PathBuf::from("/tmp/confirmed"),
        );

        let candidates = engine.discover(&make_table());

        // GB150.4-9.NDT 的 applicability 是 element_type == "外压圆筒"，应被排除
        let ndt: Vec<_> = candidates.iter().filter(|c| c.rule.id == "GB150.4-9.NDT").collect();
        assert!(ndt.is_empty(), "不适用的规则不应出现在候选列表中");
    }

    #[test]
    fn test_group_rules_by_standard() {
        let rules = vec![
            SpecRule {
                id: "GB150.3-5.3.sigma".into(),
                clause: "5.3".into(),
                severity: crate::model::Severity::Error,
                applicability: None,
                assertion: crate::rule::Assertion::Ge(
                    crate::rule::Operand::Num(0.0),
                    crate::rule::Operand::Num(0.0),
                ),
                message: None,
            },
            SpecRule {
                id: "GB150.4-7.PWHT".into(),
                clause: "7.3".into(),
                severity: crate::model::Severity::Error,
                applicability: None,
                assertion: crate::rule::Assertion::Ge(
                    crate::rule::Operand::Num(0.0),
                    crate::rule::Operand::Num(0.0),
                ),
                message: None,
            },
        ];

        let groups = group_rules_by_standard(&rules).unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("GB150.3"));
        assert!(groups.contains_key("GB150.4"));
    }
}
