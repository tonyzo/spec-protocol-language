//! 置信度计算模块
//!
//! 计算三层置信度: OCR置信度、LLM提取置信度、综合置信度

use crate::models::Confidence;
use serde::{Deserialize, Serialize};

/// 三层置信度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripleConfidence {
    /// OCR识别置信度(0.0-1.0)
    pub ocr_confidence: f64,
    /// LLM提取置信度(0.0-1.0)
    pub llm_confidence: f64,
    /// 格式校验得分(0.0-1.0)
    pub format_score: f64,
    /// 综合提取置信度(0.0-1.0)
    pub extraction_confidence: f64,
    /// 人工评估置信度(high/medium/low)
    pub manual_confidence: Confidence,
}

impl TripleConfidence {
    /// 创建新的三层置信度
    pub fn new(
        ocr_confidence: f64,
        llm_confidence: f64,
        format_score: f64,
        manual_confidence: Confidence,
    ) -> Self {
        // 计算综合置信度
        // extraction_confidence = 0.4 × OCR + 0.4 × LLM + 0.2 × 格式
        let extraction_confidence = 0.4 * ocr_confidence 
            + 0.4 * llm_confidence 
            + 0.2 * format_score;

        TripleConfidence {
            ocr_confidence,
            llm_confidence,
            format_score,
            extraction_confidence,
            manual_confidence,
        }
    }

    /// 从OCR置信度创建
    pub fn from_ocr(ocr_confidence: f64) -> Self {
        // 默认值
        TripleConfidence::new(
            ocr_confidence,
            0.85,  // LLM默认0.85
            0.90,  // 格式默认0.90
            Confidence::High,
        )
    }

    /// 获取综合置信度等级
    pub fn get_confidence_level(&self) -> &str {
        if self.extraction_confidence >= 0.9 {
            "high"
        } else if self.extraction_confidence >= 0.7 {
            "medium"
        } else {
            "low"
        }
    }

    /// 是否需要人工确认
    pub fn needs_manual_review(&self) -> bool {
        self.extraction_confidence < 0.7
    }

    /// 转换为JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

/// 置信度计算器
pub struct ConfidenceCalculator;

impl ConfidenceCalculator {
    /// 计算OCR置信度
    pub fn calculate_ocr_confidence(ocr_result: &crate::ocr::OcrResult) -> f64 {
        // 基于OCR结果的平均置信度
        if ocr_result.blocks.is_empty() {
            return ocr_result.confidence;
        }

        let sum: f64 = ocr_result.blocks.iter()
            .map(|b| b.confidence)
            .sum();
        
        sum / ocr_result.blocks.len() as f64
    }

    /// 计算LLM提取置信度
    pub fn calculate_llm_confidence(
        extracted_terms: usize,
        expected_terms: usize,
    ) -> f64 {
        if expected_terms == 0 {
            return 0.5;
        }

        // 基于提取术语数量的置信度
        let ratio = extracted_terms as f64 / expected_terms as f64;
        
        if ratio >= 1.0 {
            0.95
        } else if ratio >= 0.8 {
            0.85
        } else if ratio >= 0.6 {
            0.75
        } else {
            0.60
        }
    }

    /// 计算格式校验得分
    pub fn calculate_format_score(
        has_term_id: bool,
        has_definition: bool,
        has_spec_mapping: bool,
        has_source_clause: bool,
    ) -> f64 {
        let mut score = 0.0;

        if has_term_id { score += 0.25; }
        if has_definition { score += 0.25; }
        if has_spec_mapping { score += 0.25; }
        if has_source_clause { score += 0.25; }

        score
    }

    /// 计算综合置信度
    pub fn calculate_extraction_confidence(
        ocr_confidence: f64,
        llm_confidence: f64,
        format_score: f64,
    ) -> f64 {
        0.4 * ocr_confidence + 0.4 * llm_confidence + 0.2 * format_score
    }
}

/// 置信度报告
pub struct ConfidenceReport {
    pub triple_confidence: TripleConfidence,
    pub recommendations: Vec<String>,
}

impl ConfidenceReport {
    /// 创建新的置信度报告
    pub fn new(triple_confidence: TripleConfidence) -> Self {
        let mut recommendations = Vec::new();

        // 生成建议
        if triple_confidence.ocr_confidence < 0.7 {
            recommendations.push("OCR置信度较低,建议检查PDF/Word质量".to_string());
        }

        if triple_confidence.llm_confidence < 0.7 {
            recommendations.push("LLM提取置信度较低,建议人工复核术语定义".to_string());
        }

        if triple_confidence.format_score < 0.8 {
            recommendations.push("格式校验得分较低,建议检查param_mapping.json格式".to_string());
        }

        if triple_confidence.needs_manual_review() {
            recommendations.push("综合置信度低于0.7,强烈建议人工确认".to_string());
        }

        ConfidenceReport {
            triple_confidence,
            recommendations,
        }
    }

    /// 生成Markdown报告
    pub fn to_markdown(&self) -> String {
        let mut content = String::new();

        content.push_str("# 置信度评估报告\n\n");

        content.push_str("## 📊 三层置信度\n\n");
        content.push_str(&format!("- **OCR识别置信度**: {:.2}\n", self.triple_confidence.ocr_confidence));
        content.push_str(&format!("- **LLM提取置信度**: {:.2}\n", self.triple_confidence.llm_confidence));
        content.push_str(&format!("- **格式校验得分**: {:.2}\n", self.triple_confidence.format_score));
        content.push_str(&format!("- **综合提取置信度**: {:.2}\n", self.triple_confidence.extraction_confidence));
        content.push_str(&format!("- **人工评估**: {:?}\n\n", self.triple_confidence.manual_confidence));

        content.push_str("## 📈 置信度等级\n\n");
        content.push_str(&format!("**等级**: {}\n\n", self.triple_confidence.get_confidence_level()));

        if self.triple_confidence.needs_manual_review() {
            content.push_str("**状态**: ⚠️ 需要人工确认\n\n");
        } else {
            content.push_str("**状态**: ✅ 自动通过\n\n");
        }

        if !self.recommendations.is_empty() {
            content.push_str("## 💡 建议\n\n");
            for rec in &self.recommendations {
                content.push_str(&format!("- {}\n", rec));
            }
            content.push_str("\n");
        }

        content
    }
}
