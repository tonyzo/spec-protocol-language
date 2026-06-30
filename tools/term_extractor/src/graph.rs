//! 知识图谱模块
//!
//! 术语之间的关系网络,描述术语的语义关系

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 关系类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// 引用: A引用B
    Reference,
    /// 计算: A由B计算得出
    Calculate,
    /// 限制: A限制B的范围
    Limit,
    /// 分类: A是B的一种
    Classify,
    /// 组成: A由B组成
    Compose,
}

impl RelationType {
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            RelationType::Reference => "引用",
            RelationType::Calculate => "计算",
            RelationType::Limit => "限制",
            RelationType::Classify => "分类",
            RelationType::Compose => "组成",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "引用" | "reference" => Some(RelationType::Reference),
            "计算" | "calculate" => Some(RelationType::Calculate),
            "限制" | "limit" => Some(RelationType::Limit),
            "分类" | "classify" => Some(RelationType::Classify),
            "组成" | "compose" => Some(RelationType::Compose),
            _ => None,
        }
    }
}

/// 术语关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermRelation {
    /// 关系类型
    pub relation_type: RelationType,
    /// 目标术语ID
    pub target: String,
    /// 关系描述
    pub description: String,
    /// 置信度(0.0-1.0)
    pub confidence: f64,
}

/// 知识图谱节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeNode {
    /// 术语ID
    pub term_id: String,
    /// 标准术语
    pub standard_term: String,
    /// 关系列表
    pub relations: Vec<TermRelation>,
}

/// 知识图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    /// 节点列表
    pub nodes: Vec<KnowledgeNode>,
    /// 关系索引(term_id -> 关系列表)
    #[serde(skip)]
    pub relation_index: HashMap<String, Vec<TermRelation>>,
}

impl KnowledgeGraph {
    /// 创建新的知识图谱
    pub fn new() -> Self {
        KnowledgeGraph {
            nodes: Vec::new(),
            relation_index: HashMap::new(),
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: KnowledgeNode) {
        // 更新索引
        self.relation_index.insert(
            node.term_id.clone(),
            node.relations.clone(),
        );
        
        self.nodes.push(node);
    }

    /// 添加关系
    pub fn add_relation(
        &mut self,
        source_term_id: &str,
        relation_type: RelationType,
        target_term_id: &str,
        description: &str,
        confidence: f64,
    ) {
        let relation = TermRelation {
            relation_type,
            target: target_term_id.to_string(),
            description: description.to_string(),
            confidence,
        };

        // 更新索引
        if let Some(relations) = self.relation_index.get_mut(source_term_id) {
            relations.push(relation.clone());
        } else {
            self.relation_index.insert(
                source_term_id.to_string(),
                vec![relation.clone()],
            );
        }

        // 更新节点
        if let Some(node) = self.nodes.iter_mut().find(|n| n.term_id == source_term_id) {
            node.relations.push(relation);
        }
    }

    /// 获取术语的关系
    pub fn get_relations(&self, term_id: &str) -> Option<&Vec<TermRelation>> {
        self.relation_index.get(term_id)
    }

    /// 获取术语的引用关系
    pub fn get_references(&self, term_id: &str) -> Vec<&TermRelation> {
        self.get_relations(term_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| r.relation_type == RelationType::Reference)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 获取术语的计算关系
    pub fn get_calculations(&self, term_id: &str) -> Vec<&TermRelation> {
        self.get_relations(term_id)
            .map(|rels| {
                rels.iter()
                    .filter(|r| r.relation_type == RelationType::Calculate)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// 生成Markdown报告
    pub fn to_markdown(&self) -> String {
        let mut content = String::new();

        content.push_str("# 知识图谱\n\n");
        content.push_str(&format!("**节点数**: {}\n\n", self.nodes.len()));

        content.push_str("## 📊 关系统计\n\n");
        
        let mut type_counts = HashMap::new();
        for node in &self.nodes {
            for relation in &node.relations {
                *type_counts.entry(relation.relation_type.as_str()).or_insert(0) += 1;
            }
        }

        for (rel_type, count) in &type_counts {
            content.push_str(&format!("- **{}**: {} 个\n", rel_type, count));
        }
        content.push_str("\n");

        content.push_str("## 🔗 术语关系详情\n\n");
        for node in &self.nodes {
            if node.relations.is_empty() {
                continue;
            }

            content.push_str(&format!("### {}\n\n", node.standard_term));
            content.push_str(&format!("- **term_id**: `{}`\n", node.term_id));
            content.push_str("\n**关系**:\n\n");

            for relation in &node.relations {
                content.push_str(&format!(
                    "- **{}** → `{}`: {} (置信度: {:.2})\n",
                    relation.relation_type.as_str(),
                    relation.target,
                    relation.description,
                    relation.confidence
                ));
            }
            content.push_str("\n");
        }

        content
    }

    /// 导出为JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// 知识图谱构建器
pub struct KnowledgeGraphBuilder;

impl KnowledgeGraphBuilder {
    /// 从param_mapping.json构建知识图谱
    pub fn build_from_param_mapping(
        mapping: &crate::models::ParamMapping,
    ) -> KnowledgeGraph {
        let mut graph = KnowledgeGraph::new();

        // 创建节点
        for term in &mapping.terms {
            let node = KnowledgeNode {
                term_id: term.term_id.clone(),
                standard_term: term.standard_term.clone(),
                relations: Vec::new(),
            };
            graph.add_node(node);
        }

        // 添加关系(基于related_terms)
        for term in &mapping.terms {
            for related in &term.related_terms {
                // 推断关系类型
                let relation_type = Self::infer_relation_type(&term.term_id, related);
                
                graph.add_relation(
                    &term.term_id,
                    relation_type,
                    related,
                    &format!("{} 与 {} 相关", term.standard_term, related),
                    0.8, // 默认置信度
                );
            }
        }

        graph
    }

    /// 推断关系类型
    fn infer_relation_type(source: &str, target: &str) -> RelationType {
        // 基于术语名称推断关系类型
        if source.contains("pressure") && target.contains("pressure") {
            RelationType::Calculate
        } else if source.contains("thickness") && target.contains("thickness") {
            RelationType::Compose
        } else if source.contains("stress") && target.contains("stress") {
            RelationType::Limit
        } else if source.contains("temperature") && target.contains("temperature") {
            RelationType::Reference
        } else {
            RelationType::Reference
        }
    }
}
