# RVM Model Checker & SPEC Protocol

将工程标准（GB150、NB47012、ASME VIII）整理为机器可读的规则库，并实现标准术语提取、参数映射、跨标准冲突检测和合规性验证。

## 能做什么

### 标准术语提取与参数映射
从标准文档（PDF/Word）中提取工程参数，建立"标准原文术语 → SPEC 统一参数名"的映射表，支持自动识别参数名、单位、限值和定义。

### 规则库生成
将提取的参数映射转化为 SPEC Protocol 可消费的 YAML 规则文件，包含 `assert`（断言表达式）和 `limit`（限值定义），支持数值比较、范围检查和公式计算。

### 跨标准冲突检测
自动比对多个标准中同名参数的术语、定义和单位，发现三类冲突：
- **术语冲突**（Info）：同一概念术语不同，已映射无需处理
- **参数冲突**（Warning）：同一参数定义不同，需人工确认
- **单位冲突**（Warning）：同一参数单位不同，附转换建议

### 批量处理
通过 JSON 配置文件一次处理多个标准，自动执行完整链路：OCR 预处理 → 术语提取 → 三层置信度计算 → 知识图谱构建 → 跨标准冲突检测。

### 合规性验证
对规则 YAML 做结构校验和量纲检查，确保规则文件可被下游 RVM Model Checker 正确消费。

### 参数映射差异比对
比较同一标准的两个版本 param_mapping.json，输出新增/删除/修改的术语清单。

## 快速开始

### 构建

```bash
cd tools/term_extractor
cargo build --release
```

### CLI 命令一览

```bash
# 1. 从标准文档提取术语，生成 param_mapping.json
term_extractor extract -i GB150.3.pdf -o output/GB150.3 --standard "GB150.3"

# 2. 从 param_mapping.json 生成规则 YAML
term_extractor generate-rules -i output/GB150.3/param_mapping.json -o rules/library/GB150.3/

# 3. 验证规则 YAML 的结构和量纲
term_extractor validate -i rules/library/GB150.3/5.3-内压圆筒.yaml

# 4. 比较两个版本的参数映射差异
term_extractor diff -o old/param_mapping.json -n new/param_mapping.json -r diff_report.md

# 5. 生成标准文档术语表（Markdown）
term_extractor generate-docs -i output/GB150.3/param_mapping.json -o glossary.md

# 6. OCR 预处理（PDF/Word → 结构化文本）
term_extractor ocr -i standard.pdf -o output/structured.md

# 7. 批量提取（多标准并行处理）
term_extractor batch-extract --config batch_config.json -o output/

# 8. 跨标准冲突检测
term_extractor detect-conflicts --standards output/GB150.3 output/NB47012 -o conflict_report.md
```

### 批量处理配置示例

```json
{
  "standards": [
    { "name": "GB150.3", "path": "standards/GB150.3.pdf" },
    { "name": "NB47012", "path": "standards/NB47012.pdf" }
  ],
  "options": {
    "ocr_enabled": true,
    "confidence_enabled": true,
    "graph_enabled": true,
    "conflict_detection": true,
    "parallel": true,
    "max_threads": 4
  }
}
```

## 项目结构

```
.
├── tools/term_extractor/          # 核心工具（Rust）
│   └── src/
│       ├── main.rs                # CLI 入口，8 个子命令
│       ├── extract.rs             # 术语提取引擎
│       ├── ocr.rs                 # OCR 预处理（PDF/Word → 结构化文本）
│       ├── confidence.rs          # 三层置信度计算（OCR + LLM + 格式）
│       ├── graph.rs               # 知识图谱构建（5 种关系类型）
│       ├── batch.rs               # 批量处理（并行 + 自动冲突检测）
│       ├── conflicts/             # 冲突检测深模块
│       │   ├── mod.rs             # ConflictDetector（一次索引，多种检测）
│       │   └── report.rs          # 参数化报告生成
│       ├── diff.rs                # 参数映射差异比对
│       ├── validate.rs            # 规则 YAML 校验
│       ├── docs.rs                # 术语表生成
│       ├── rules.rs               # 规则 YAML 生成
│       └── models.rs              # 数据模型
├── rules/library/                 # 已整理的规则库
│   ├── GB150.1/                   # 通用要求
│   ├── GB150.2/                   # 材料
│   ├── GB150.3/                   # 设计（内压圆筒、球壳、封头、开孔补强等）
│   ├── GB150.4/                   # 制造（焊接、热处理、无损检测、耐压试验）
│   ├── NB47012/                   # NB/T 47012 制造
│   └── ASME_VIII/                 # ASME Boiler & Pressure Vessel Code
├── crates/spec_core/              # SPEC Protocol 核心库
├── docs/adr/                      # 架构决策记录（8 份）
├── CONTEXT.md                     # 统一语言文档
└── GB150.1-4/                     # 标准原文（Word）
```

## 已整理的规则库

| 标准 | 规则文件数 | 覆盖范围 |
|------|-----------|---------|
| GB150.1 | 1 | 通用要求 |
| GB150.2 | 1 | 材料总体要求 |
| GB150.3 | 9 | 内压圆筒、球壳、封头、开孔补强、法兰、焊接、无损检测、耐压试验、外压 |
| GB150.4 | 5 | 焊接、热处理、无损检测、耐压试验 |
| NB47012 | — | 参数映射已就绪 |
| ASME VIII | — | 参数映射已就绪 |

## 架构决策记录（ADR）

| 编号 | 主题 |
|------|------|
| ADR-001 | 职责边界 — 只做 compliance |
| ADR-002 | 核心语言 — Rust + TS SDK |
| ADR-003 | 输入契约 — 结构化参数表 |
| ADR-004 | 报告格式 — RVM 协作格式对接 PDMS |
| ADR-005 | 量纲校验与参数表分区 |
| ADR-006 | 审查点生命周期管理 |
| ADR-007 | 标准术语提取与参数映射工具 |
| ADR-008 | OCR 集成与批量处理架构 |

## 技术栈

- **语言**: Rust 2021 Edition
- **CLI**: clap v4
- **序列化**: serde (JSON + YAML)
- **文本处理**: regex, pulldown-cmark, similar
- **并行**: std::thread + Arc\<Mutex\>

## 开发状态

| 功能 | 状态 |
|------|------|
| 术语提取与参数映射 | ✅ 可用 |
| 规则 YAML 生成 | ✅ 可用 |
| 规则校验与量纲检查 | ✅ 可用 |
| 差异比对 | ✅ 可用 |
| 术语表生成 | ✅ 可用 |
| OCR 预处理 | ⚠️ 模拟数据，待集成真实 OCR |
| 三层置信度 | ⚠️ OCR/LLM 模块为模拟值 |
| 知识图谱 | ✅ 可用 |
| 批量处理 | ✅ 可用 |
| 冲突检测 | ✅ 可用 |
