# term_extractor 使用指南

> **标准术语提取与参数映射工具** - 将工程标准(PDF/Word/Markdown)整理为SPEC Protocol可用的术语表和参数映射

---

## 快速开始

### 1. 编译工具

```bash
cd tools/term_extractor
cargo build --release
```

编译后的可执行文件位于: `target/release/term_extractor.exe`

### 2. 查看帮助

```bash
term_extractor --help
```

输出:
```
将工程标准(PDF/Word/Markdown)整理为SPEC Protocol可用的术语表和参数映射

Usage: term_extractor.exe [OPTIONS] <COMMAND>

Commands:
  extract         从标准原文提取术语
  generate-docs   从param_mapping.json生成术语定义表Markdown文档
  generate-rules  从param_mapping.json生成规则YAML骨架
  diff            比对两个版本的参数映射差异
  validate        校验参数映射的一致性
  help            Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose  启用详细日志
  -h, --help     Print help
```

---

## 核心命令

### 命令1: generate-docs (生成术语文档)

**功能**: 从`param_mapping.json`生成术语定义表Markdown文档

**用法**:
```bash
term_extractor generate-docs \
  --mapping rules/library/GB150/param_mapping.json \
  --output docs/GB150-术语定义表.md
```

**输入**: `param_mapping.json` (手动编辑或自动提取)

**输出**: 结构化的术语定义表Markdown文档,包含:
- 按类别分组的术语表格(压力/温度/厚度/应力/几何/材料)
- 分类属性术语表
- 参数分组速查
- 跨标准引用关系
- 校验规则

**示例**:
```bash
# 生成GB150术语文档
term_extractor generate-docs \
  --mapping rules/library/GB150/param_mapping.json \
  --output docs/GB150-术语定义与概念映射.md
```

---

### 命令2: extract (提取术语 - 待实现)

**功能**: 从标准原文(PDF/Word/Markdown)自动提取术语

**用法**:
```bash
term_extractor extract \
  --input standards/NB_T_47012-2023.pdf \
  --output rules/library/NB47012/ \
  --standard NB47012 \
  --version 2023 \
  --parts "NB/T 47012-2023 换热压力容器"
```

**输入**: 标准原文文件(PDF/Word/Markdown)

**输出**:
- `rules/library/{标准号}/param_mapping.json` - 参数映射表
- `docs/{标准号}-术语定义与概念映射.md` - 术语文档(自动生成)
- `docs/{标准号}-条款引用关系.md` - 引用关系图

**工作流程**:
1. 解析标准文档,定位"术语和定义"章节
2. 提取术语名称、定义、条款号
3. 自动分类(parameter/limit/attr)
4. 生成param_mapping.json
5. 生成Markdown文档

**状态**: ⏳ 待实现(Phase 2)

---

### 命令3: generate-rules (生成规则YAML骨架 - 待实现)

**功能**: 从param_mapping.json生成规则YAML文件骨架

**用法**:
```bash
term_extractor generate-rules \
  --mapping rules/library/GB150/param_mapping.json \
  --template rules/.templates/ \
  --output rules/library/GB150.3/
```

**输入**: param_mapping.json + 规则模板

**输出**: 规则YAML文件骨架,参数名已自动替换

**示例输出**:
```yaml
# GB150.3-5.3-内压圆筒.yaml
- id: GB150.3-5.3.sigma
  clause: "5.3 / 公式 5-5"
  severity: error
  applicability:
    type: eq
    value:
      - {attr: element_type}
      - "内压圆筒"
  assertion:
    type: le
    value:
      - {param: sigma}          # 自动从param_mapping.json映射
      - {limit: sigma_allow}    # 自动从param_mapping.json映射
  message: "环向应力 {actual} MPa 超过许用应力 {expected} MPa"
```

**状态**: ⏳ 待实现(Phase 2)

---

### 命令4: diff (比对术语变更 - 待实现)

**功能**: 比对两个版本的参数映射差异

**用法**:
```bash
term_extractor diff \
  --old rules/library/GB150-2011/param_mapping.json \
  --new rules/library/GB150-2024/param_mapping.json \
  --output docs/GB150-术语变更差异.md
```

**输出**: 差异报告,包含:
- 新增术语
- 删除术语
- 定义变更
- 参数映射变更

**状态**: ⏳ 待实现(Phase 3)

---

### 命令5: validate (校验一致性 - 待实现)

**功能**: 校验参数映射的一致性

**用法**:
```bash
term_extractor validate \
  --rules rules/library/ \
  --report docs/术语一致性报告.md
```

**检查项**:
- YAML中的参数是否在param_mapping.json中定义
- 单位一致性
- 术语完整性
- 命名规范

**状态**: ⏳ 待实现(Phase 3)

---

## 完整工作流程

### 场景1: 新标准接入(手动整理)

```bash
# 1. 创建目录
mkdir -p rules/library/NB47012
mkdir -p docs

# 2. 手动编辑param_mapping.json
# 参考: rules/library/GB150/param_mapping.json

# 3. 生成术语文档
term_extractor generate-docs \
  --mapping rules/library/NB47012/param_mapping.json \
  --output docs/NB47012-术语定义与概念映射.md

# 4. 人工校对术语文档
# 打开 docs/NB47012-术语定义与概念映射.md 检查

# 5. (待实现)生成规则YAML骨架
# term_extractor generate-rules \
#   --mapping rules/library/NB47012/param_mapping.json \
#   --template rules/.templates/ \
#   --output rules/library/NB47012/
```

### 场景2: 标准更新比对(待实现)

```bash
# 比对GB150-2011和GB150-2024
term_extractor diff \
  --old rules/library/GB150-2011/param_mapping.json \
  --new rules/library/GB150-2024/param_mapping.json \
  --output docs/GB150-术语变更差异.md
```

### 场景3: 批量校验(待实现)

```bash
# 校验所有标准的术语映射一致性
term_extractor validate \
  --rules rules/library/ \
  --report docs/术语一致性报告.md
```

---

## param_mapping.json 格式说明

### 核心结构

```json
{
  "_schema_version": "1.0",
  "_description": "GB150 参数映射表",
  "_standard": "GB/T 150.1~150.4-2024",
  
  "standard_info": {
    "name": "GB/T 150 压力容器",
    "version": "2024",
    "parts": ["GB/T 150.1-2024 通用要求", "..."]
  },

  "terms": [
    {
      "term_id": "pressure.design",
      "standard_term": "设计压力",
      "definition": "设定的容器顶部的最高压力,不低于工作压力",
      "spec_mapping": {
        "type": "parameter",
        "name": "design_pressure",
        "unit": "MPa",
        "data_type": "f64"
      },
      "source_clause": "3.1.4",
      "referenced_by": ["GB150.3-5.3"],
      "related_terms": ["pressure.operating"],
      "confidence": "high"
    }
  ],

  "parameter_groups": {
    "design": ["design_pressure", "design_temperature"],
    "geometry": ["D_i", "D_o", "delta"],
    "stress": ["sigma", "sigma_allow"],
    "material": ["R_m", "R_eL"],
    "welding_ndt": ["phi", "ndt_ratio"],
    "test": ["test_pressure_actual"],
    "limits": ["delta_min", "MAWP"],
    "attrs": ["element_type", "material_category"]
  },

  "cross_references": {
    "GB150.3→GB150.2": {
      "description": "设计计算引用许用应力",
      "terms": ["sigma_allow", "R_m"]
    }
  },

  "validation_rules": [
    {
      "rule": "所有parameters必须携带unit字段",
      "check": "terms[*].spec_mapping.type=='parameter' implies spec_mapping.unit != null"
    }
  ]
}
```

### 字段说明

| 字段 | 类型 | 必填 | 说明 |
|-----|------|------|------|
| `term_id` | string | ✅ | 术语ID(英文点分,如`pressure.design`) |
| `standard_term` | string | ✅ | 标准原文术语 |
| `definition` | string | ✅ | 术语定义 |
| `spec_mapping.type` | enum | ✅ | parameter/limit/attr |
| `spec_mapping.name` | string | ✅ | SPEC参数名(蛇形命名) |
| `spec_mapping.unit` | string | ⚠️ | 单位(attrs为null) |
| `spec_mapping.data_type` | string | ✅ | f64/string/bool |
| `spec_mapping.allowed_values` | array | ⚠️ | 仅attr需要,枚举值 |
| `source_clause` | string | ✅ | 来源条款号 |
| `referenced_by` | array | ❌ | 哪些规则引用此术语 |
| `related_terms` | array | ❌ | 相关术语 |
| `confidence` | enum | ✅ | high/medium/low |
| `note` | string | ❌ | 补充说明 |
| `synonyms` | array | ❌ | 同义词 |

---

## 参数分类原则

| 类型 | 判断标准 | 示例 |
|-----|---------|------|
| **parameter** | 计算软件提供的计算值 | sigma(应力), delta(厚度), design_pressure |
| **limit** | 标准规定的限值/阈值 | delta_min(最小壁厚), sigma_allow(许用应力) |
| **attr** | 分类属性(字符串,用于适用性筛选) | element_type, material_category, medium_toxicity |

**判断口诀**:
- 如果是"计算出来的值" → `parameter`
- 如果是"标准规定的限值" → `limit`
- 如果是"类型/状态/标志" → `attr`

---

## 命名规范

| 类型 | 规范 | 示例 |
|-----|------|------|
| **term_id** | 英文点分,分类.术语 | `pressure.design`, `thickness.calc` |
| **spec_mapping.name** | 蛇形命名(全小写+下划线) | `design_pressure`, `sigma_allow` |
| **标准术语** | 保留标准原文 | "设计压力", "许用应力" |

---

## 当前实现状态

| 功能 | 状态 | 说明 |
|-----|------|------|
| **generate-docs** | ✅ 已实现 | 从param_mapping.json生成Markdown文档 |
| **extract** | ⏳ 待实现 | PDF/Word解析+LLM辅助提取 |
| **generate-rules** | ⏳ 待实现 | 规则YAML骨架生成 |
| **diff** | ⏳ 待实现 | 术语变更比对 |
| **validate** | ⏳ 待实现 | 一致性校验 |

---

## 下一步计划

### Phase 1 (MVP) - ✅ 已完成
- [x] 定义数据结构
- [x] 实现CLI框架
- [x] 实现generate-docs命令
- [x] 编写使用指南

### Phase 2 (自动化) - 进行中
- [ ] 实现extract命令(PDF/Word解析)
- [ ] 实现generate-rules命令
- [ ] LLM辅助术语提取

### Phase 3 (集成)
- [ ] 实现diff命令
- [ ] 实现validate命令
- [ ] 批量一致性校验

---

**文档版本**: 1.0  
**创建日期**: 2025-06-29  
**维护者**: SPEC Protocol 团队  
**工具版本**: term_extractor v0.1.0
