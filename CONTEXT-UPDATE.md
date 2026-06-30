# SPEC Protocol — 统一语言 (Ubiquitous Language) 更新

> 本文档记录ADR-008新增的术语

---

## 新增术语

### OCR相关

#### Unlimited OCR
百度开源的端到端文档解析模型,总参数3B,激活参数570M。
- 支持: 单图/多页长文档/PDF直读/批量处理
- 性能: OmniDocBench v1.6 综合得分93.92%
- 特性: 置信度评估(0.0-1.0)

#### OCR Confidence (OCR置信度)
OCR模型对文本识别的置信度分数,范围0.0-1.0。
- 0.9-1.0: 高置信度(识别准确)
- 0.7-0.9: 中置信度(可能需要人工确认)
- < 0.7: 低置信度(需要人工确认)

#### Extraction Confidence (提取置信度)
术语提取的综合置信度,由三部分组成:
```
extraction_confidence = 
  0.4 × ocr_confidence +           // OCR识别准确性
  0.4 × llm_extraction_confidence + // LLM术语提取置信度
  0.2 × format_validation_score     // 格式校验得分
```

---

### 知识图谱相关

#### Knowledge Graph (知识图谱)
术语之间的关系网络,描述术语的语义关系。

**关系类型**:
- 引用(reference): A引用B
- 计算(calculate): A由B计算得出
- 限制(limit): A限制B的范围
- 分类(classify): A是B的一种
- 组成(compose): A由B组成

**示例**:
```json
{
  "term_id": "pressure.design",
  "relations": [
    {
      "type": "引用",
      "target": "pressure.operating",
      "description": "设计压力基于工作压力确定"
    }
  ]
}
```

---

### 批量处理相关

#### Batch Processing (批量处理)
同时处理多个标准或多个文件的能力。

**配置示例**:
```json
{
  "standards": [
    {
      "name": "GB150",
      "version": "2024",
      "files": ["GB_T 150.1-2024.pdf", ...]
    }
  ],
  "options": {
    "parallel": true,
    "ocr_enabled": true,
    "conflict_detection": true
  }
}
```

---

### 冲突检测相关

#### Terminology Conflict (术语冲突)
同一概念在不同标准中术语不同。

**示例**:
- GB150: "设计压力"
- ASME VIII: "Design Pressure"

**严重度**: info(已映射到同一SPEC参数)

---

#### Parameter Conflict (参数冲突)
同一参数在不同标准中定义不同。

**示例**:
- GB150: δ_min (最小壁厚,考虑腐蚀裕量)
- ASME VIII: Required Thickness (所需厚度,由公式计算)

**严重度**: warning(需要人工确认)

---

#### Rule Conflict (规则冲突)
同一场景在不同标准中要求不同。

**示例**:
- GB150: φ ≤ 1.0 (焊接接头系数)
- ASME VIII: E ≤ 1.0 (Joint Efficiency)

**严重度**: error(要求一致,但参数名不同)

---

#### Unit Conflict (单位冲突)
同一参数在不同标准中单位不同。

**示例**:
- GB150: design_pressure → MPa
- ASME VIII: design_pressure → psi

**严重度**: warning(需要单位转换)

---

#### Conflict Detection (冲突检测)
自动检测跨标准的术语/参数/规则/单位冲突的能力。

**命令**:
```bash
term_extractor detect-conflicts \
  --standards rules/library/GB150/ rules/library/ASME_VIII/ \
  --output docs/冲突检测报告.md \
  --types all
```

---

## 更新后的术语统计

| 类别 | 新增 | 总计 |
|-----|------|------|
| **OCR相关** | 3 | 3 |
| **知识图谱** | 2 | 2 |
| **批量处理** | 1 | 1 |
| **冲突检测** | 5 | 5 |
| **总计** | **11** | **11** |

---

**更新日期**: 2025-06-30  
**更新原因**: ADR-008 实施
