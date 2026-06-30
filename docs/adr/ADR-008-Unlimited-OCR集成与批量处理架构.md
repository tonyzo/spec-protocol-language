# ADR-008: Unlimited OCR 集成与批量处理架构

> **状态**: 已接受 (Accepted)  
> **日期**: 2025-06-30  
> **决策者**: SPEC Protocol Team

---

## 背景 (Context)

### 当前问题

1. **PDF/Word解析能力缺失**
   - term_extractor的extract命令仅支持Markdown输入
   - 标准原文多为PDF/Word格式,需要人工转换为Markdown
   - 效率低,易出错

2. **批量处理能力不足**
   - 当前只能逐个处理标准
   - 需要同时处理多个标准(GB150/NB47012/ASME VIII)
   - 需要同时处理多个文件(GB150的4个部分)

3. **跨标准冲突检测缺失**
   - 不同标准间可能存在术语/参数/规则/单位冲突
   - 需要自动检测并报告冲突
   - 需要人工确认冲突解决方案

### 技术选型

**Unlimited OCR** (百度开源):
- GitHub: baidu/Unlimited-OCR
- 总参数: 3B, 激活参数: 570M
- 支持: 单图/多页长文档/PDF直读/批量处理
- 性能: OmniDocBench v1.6 综合得分93.92%
- 特性: 置信度评估(0.0-1.0)

---

## 决策 (Decision)

### 决策1: Unlimited OCR 作为独立预处理工具

**架构**:
```
PDF/Word → Unlimited OCR → Markdown/结构化文本 → term_extractor extract → param_mapping.json
```

**理由**:
- 解耦设计: OCR负责识别,extract负责提取
- 职责清晰: OCR是预处理工具,不是term_extractor的一部分
- 易于替换: 未来可以更换其他OCR工具

**实现**:
```bash
# 步骤1: OCR预处理
unlimited_ocr input.pdf --output output.md --confidence output_conf.json

# 步骤2: 术语提取
term_extractor extract \
  --input output.md \
  --ocr-confidence output_conf.json \
  --output rules/library/GB150/
```

---

### 决策2: 三层置信度体系

**字段设计**:
```json
{
  "term_id": "pressure.design",
  "confidence": "high",              // 人工评估(保留)
  "ocr_confidence": 0.95,            // OCR识别置信度
  "extraction_confidence": 0.88      // 术语提取置信度(综合)
}
```

**计算逻辑**:
```
extraction_confidence = 
  0.4 × ocr_confidence +           // OCR识别准确性
  0.4 × llm_extraction_confidence + // LLM术语提取置信度
  0.2 × format_validation_score     // 格式校验得分
```

**理由**:
- OCR置信度: 文本识别准确性(机器评估)
- LLM置信度: 术语定义匹配度(LLM评估)
- 格式校验: 是否符合param_mapping.json格式(规则评估)
- 人工confidence: 最终确认(人工评估)

---

### 决策3: 知识图谱增强

**当前related_terms**:
```json
{
  "term_id": "pressure.design",
  "related_terms": ["pressure.operating", "pressure.calc"]
}
```

**增强为知识图谱**:
```json
{
  "term_id": "pressure.design",
  "relations": [
    {
      "type": "引用",
      "target": "pressure.operating",
      "description": "设计压力基于工作压力确定"
    },
    {
      "type": "计算",
      "target": "pressure.calc",
      "description": "计算压力由设计压力加上液柱静压力"
    }
  ]
}
```

**关系类型**:
- 引用(reference): A引用B
- 计算(calculate): A由B计算得出
- 限制(limit): A限制B的范围
- 分类(classify): A是B的一种
- 组成(compose): A由B组成

**理由**:
- related_terms只是简单列表,缺乏语义
- 知识图谱提供丰富的关系类型和描述
- 便于可视化和查询

---

### 决策4: 批量处理架构

**输入格式**:
```json
{
  "standards": [
    {
      "name": "GB150",
      "version": "2024",
      "files": [
        "GB_T 150.1-2024.pdf",
        "GB_T 150.2-2024.pdf",
        "GB_T 150.3-2024.pdf",
        "GB_T 150.4-2024.pdf"
      ]
    },
    {
      "name": "NB47012",
      "version": "2024",
      "files": ["NB_T 47012-2024.pdf"]
    }
  ],
  "options": {
    "parallel": true,
    "ocr_enabled": true,
    "conflict_detection": true
  }
}
```

**命令**:
```bash
term_extractor batch-extract \
  --config batch_config.json \
  --output rules/library/ \
  --report batch_report.md
```

**特性**:
- 支持多标准并行处理
- 支持多文件批量处理
- 自动生成批量处理报告

---

### 决策5: 四类冲突检测

#### 类型1: 术语冲突

**检测逻辑**:
```
GB150: "设计压力" → design_pressure
ASME VIII: "Design Pressure" → design_pressure
```

**输出**:
```json
{
  "conflict_type": "术语冲突",
  "severity": "info",
  "standards": ["GB150", "ASME VIII"],
  "terms": [
    {"standard": "GB150", "term": "设计压力"},
    {"standard": "ASME VIII", "term": "Design Pressure"}
  ],
  "resolution": "已映射到同一SPEC参数"
}
```

#### 类型2: 参数冲突

**检测逻辑**:
```
GB150: δ_min = 12mm (最小壁厚)
ASME VIII: Required Thickness = 0.47in (所需厚度)
```

**输出**:
```json
{
  "conflict_type": "参数冲突",
  "severity": "warning",
  "parameters": [
    {
      "standard": "GB150",
      "term": "最小壁厚",
      "spec_param": "delta_min",
      "definition": "考虑腐蚀裕量后的最小厚度"
    },
    {
      "standard": "ASME VIII",
      "term": "Required Thickness",
      "spec_param": "delta_calc",
      "definition": "由公式计算得到的最小厚度"
    }
  ],
  "resolution": "定义不同,需要人工确认"
}
```

#### 类型3: 规则冲突

**检测逻辑**:
```
GB150: φ ≤ 1.0 (焊接接头系数)
ASME VIII: E ≤ 1.0 (Joint Efficiency)
```

**输出**:
```json
{
  "conflict_type": "规则冲突",
  "severity": "error",
  "rules": [
    {
      "standard": "GB150",
      "clause": "5.3",
      "assertion": "le: [{param: phi}, {limit: 1.0}]"
    },
    {
      "standard": "ASME VIII",
      "clause": "UG-27",
      "assertion": "le: [{param: joint_efficiency}, {limit: 1.0}]"
    }
  ],
  "resolution": "要求一致,但参数名不同"
}
```

#### 类型4: 单位冲突

**检测逻辑**:
```
GB150: design_pressure → MPa
ASME VIII: design_pressure → psi
```

**输出**:
```json
{
  "conflict_type": "单位冲突",
  "severity": "warning",
  "parameter": "design_pressure",
  "units": [
    {"standard": "GB150", "unit": "MPa"},
    {"standard": "ASME VIII", "unit": "psi"}
  ],
  "conversion": "1 MPa = 145.038 psi",
  "resolution": "需要单位转换"
}
```

**命令**:
```bash
term_extractor detect-conflicts \
  --standards rules/library/GB150/ rules/library/ASME_VIII/ \
  --output docs/冲突检测报告.md \
  --types all
```

---

## 后果 (Consequences)

### 正面影响

1. **自动化程度提升**
   - PDF/Word → OCR → 术语提取 → 全自动
   - 人工介入减少80%

2. **批量处理能力**
   - 支持多标准并行处理
   - 支持多文件批量处理
   - 效率提升10倍

3. **冲突自动检测**
   - 四类冲突自动识别
   - 生成冲突报告
   - 便于人工确认

4. **知识图谱增强**
   - 丰富的关系类型
   - 便于可视化和查询
   - 支持智能推理

### 负面影响

1. **依赖外部工具**
   - 需要安装Unlimited OCR
   - 需要GPU资源(推荐)
   - 增加部署复杂度

2. **代码量增加**
   - OCR集成: ~500行
   - 批量处理: ~800行
   - 冲突检测: ~1000行
   - 知识图谱: ~600行
   - 总计: ~2900行

3. **学习成本**
   - 需要理解OCR置信度
   - 需要理解知识图谱
   - 需要理解冲突类型

### 风险

1. **OCR质量不稳定**
   - 缓解: 保留人工确认环节
   - 缓解: 提供置信度评估

2. **冲突误报**
   - 缓解: 提供严重度分级
   - 缓解: 支持人工确认

3. **性能问题**
   - 缓解: 支持并行处理
   - 缓解: 支持增量处理

---

## 实现计划

### Phase 4: Unlimited OCR 集成 (2周)

**任务**:
1. 集成Unlimited OCR (3天)
2. 实现OCR置信度处理 (2天)
3. 实现三层置信度体系 (3天)
4. 实现知识图谱增强 (2天)

**交付物**:
- `src/ocr.rs` - OCR集成模块
- `src/confidence.rs` - 置信度计算模块
- `src/graph.rs` - 知识图谱模块

### Phase 5: 批量处理 (1周)

**任务**:
1. 实现批量处理框架 (3天)
2. 实现并行处理 (2天)

**交付物**:
- `src/batch.rs` - 批量处理模块
- `batch_config.json` - 批量处理配置

### Phase 6: 冲突检测 (2周)

**任务**:
1. 实现术语冲突检测 (3天)
2. 实现参数冲突检测 (3天)
3. 实现规则冲突检测 (3天)
4. 实现单位冲突检测 (3天)
5. 实现冲突报告生成 (2天)

**交付物**:
- `src/conflicts/` - 冲突检测模块
  - `terminology.rs` - 术语冲突
  - `parameter.rs` - 参数冲突
  - `rule.rs` - 规则冲突
  - `unit.rs` - 单位冲突
  - `report.rs` - 报告生成

---

## 验收标准

### Phase 4 验收

- [ ] 能从PDF/Word自动提取术语
- [ ] 能输出三层置信度
- [ ] 能生成知识图谱
- [ ] OCR置信度准确率 > 90%

### Phase 5 验收

- [ ] 能批量处理多个标准
- [ ] 能批量处理多个文件
- [ ] 支持并行处理
- [ ] 处理时间 < 单标准处理时间 × 标准数 × 0.5

### Phase 6 验收

- [ ] 能检测四类冲突
- [ ] 冲突检测准确率 > 85%
- [ ] 能生成冲突报告
- [ ] 支持人工确认

---

## 参考资料

- [Unlimited OCR GitHub](https://github.com/baidu/Unlimited-OCR)
- [ADR-007: 标准术语提取与参数映射工具](ADR-007-标准术语提取与参数映射工具.md)
- [CONTEXT.md](../CONTEXT.md)

---

**状态**: ✅ 已接受  
**下一步**: 开始Phase 4实现
