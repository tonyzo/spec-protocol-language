//! 规则库 + 注册表管理 (ADR-006)
//!
//! - RuleLibrary: 从 `rules/library/` 目录递归加载所有标准的全部 SpecRule。
//! - ReviewPointRegistry: 从 `rules/registry.json` 加载/保存审查点注册表。

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::{RegistryEntry, ReviewPointRegistry};
use crate::rule::SpecRule;

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML 解析错误: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("路径无效: {0}")]
    Path(String),
    #[error("规则库目录不存在: {0}")]
    LibraryNotFound(String),
}

/// 规则库：从 `rules/library/` 目录加载所有标准的全部规则。
pub struct RuleLibrary {
    /// 全部规则（按加载顺序）
    rules: Vec<SpecRule>,
    /// 规则 → 来源文件路径的映射（用于溯源）
    source_map: HashMap<String, PathBuf>,
}

impl RuleLibrary {
    /// 从目录递归加载所有 .yaml 文件中的规则。
    ///
    /// 目录结构示例:
    /// ```text
    /// rules/library/
    /// ├── GB150.3/
    /// │   └── 5.3-内压圆筒.yaml
    /// └── GB150.4/
    ///     ├── 7-热处理.yaml
    ///     └── 9-无损检测.yaml
    /// ```
    pub fn from_dir(dir: &Path) -> Result<Self, RegistryError> {
        if !dir.exists() {
            return Err(RegistryError::LibraryNotFound(
                dir.display().to_string(),
            ));
        }

        let mut rules = Vec::new();
        let mut source_map = HashMap::new();
        let mut yaml_files = Vec::new();
        collect_yaml_files(dir, &mut yaml_files)?;

        // 按文件路径排序，确保加载顺序稳定
        yaml_files.sort();

        for file_path in &yaml_files {
            let yaml_text = fs::read_to_string(file_path)?;
            let file_rules: Vec<SpecRule> = serde_yaml::from_str(&yaml_text)?;
            for rule in file_rules {
                source_map.insert(rule.id.clone(), file_path.clone());
                rules.push(rule);
            }
        }

        Ok(Self {
            rules,
            source_map,
        })
    }

    /// 从单段 YAML 文本加载（用于测试或单文件场景）。
    pub fn from_yaml(yaml: &str) -> Result<Self, RegistryError> {
        let rules: Vec<SpecRule> = serde_yaml::from_str(yaml)?;
        Ok(Self {
            rules,
            source_map: HashMap::new(),
        })
    }

    /// 返回全部规则。
    pub fn rules(&self) -> &[SpecRule] {
        &self.rules
    }

    /// 按标准号前缀过滤规则。
    /// 例: filter_by_standard("GB150.3") 返回所有 id 以 "GB150.3" 开头的规则。
    pub fn filter_by_standard(&self, standard_prefix: &str) -> Vec<&SpecRule> {
        self.rules
            .iter()
            .filter(|r| r.id.starts_with(standard_prefix))
            .collect()
    }

    /// 查询规则的来源文件路径。
    pub fn source_of(&self, rule_id: &str) -> Option<&Path> {
        self.source_map.get(rule_id).map(|p| p.as_path())
    }

    /// 规则总数。
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

/// 递归收集目录下所有 .yaml / .yml 文件。
fn collect_yaml_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), RegistryError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files(&path, out)?;
        } else if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            out.push(path);
        }
    }
    Ok(())
}

/// 注册表文件管理。
impl ReviewPointRegistry {
    /// 从 JSON 文件加载注册表。
    pub fn load(path: &Path) -> Result<Self, RegistryError> {
        if !path.exists() {
            // 文件不存在时返回空注册表
            return Ok(Self {
                version: "1.0".into(),
                design_types: HashMap::new(),
            });
        }
        let json_text = fs::read_to_string(path)?;
        let registry: Self = serde_json::from_str(&json_text)?;
        Ok(registry)
    }

    /// 保存注册表到 JSON 文件。
    pub fn save(&self, path: &Path) -> Result<(), RegistryError> {
        let json_text = serde_json::to_string_pretty(self)?;
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, json_text)?;
        Ok(())
    }

    /// 从设计类型和注册条目构建文件完整路径。
    pub fn resolved_file_path(&self, confirmed_dir: &Path, design_type: &str, entry: &RegistryEntry) -> PathBuf {
        confirmed_dir.join(design_type).join(&entry.file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_from_yaml() {
        let yaml = r#"
- id: test.1
  clause: "1.1"
  severity: error
  assertion:
    type: ge
    value: [{param: x}, 0]
"#;
        let lib = RuleLibrary::from_yaml(yaml).unwrap();
        assert_eq!(lib.len(), 1);
        assert_eq!(lib.rules()[0].id, "test.1");
    }

    #[test]
    fn test_filter_by_standard() {
        let yaml = r#"
- id: GB150.3-5.3.sigma
  clause: "5.3"
  severity: error
  assertion:
    type: ge
    value: [{param: x}, 0]
- id: GB150.4-7.PWHT
  clause: "7.3"
  severity: error
  assertion:
    type: ge
    value: [{param: x}, 0]
"#;
        let lib = RuleLibrary::from_yaml(yaml).unwrap();
        assert_eq!(lib.len(), 2);
        assert_eq!(lib.filter_by_standard("GB150.3").len(), 1);
        assert_eq!(lib.filter_by_standard("GB150.4").len(), 1);
        assert_eq!(lib.filter_by_standard("GB150").len(), 2);
    }

    #[test]
    fn test_registry_load_missing() {
        let registry = ReviewPointRegistry::load(Path::new("/nonexistent/registry.json")).unwrap();
        assert!(!registry.has_design_type("内压圆筒"));
    }

    #[test]
    fn test_registry_add_and_query() {
        let mut registry = ReviewPointRegistry::default();
        assert!(!registry.has_design_type("内压圆筒"));

        registry.add_entry(
            "内压圆筒",
            RegistryEntry {
                standard: "GB150.3".into(),
                clause: "5.3".into(),
                file_path: "GB150.3-5.3-2024.yaml".into(),
                rule_count: 3,
                confirmed_at: "2025-06-29T10:00:00Z".into(),
            },
        );

        assert!(registry.has_design_type("内压圆筒"));
        let entries = registry.get_entries("内压圆筒").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].standard, "GB150.3");
    }
}
