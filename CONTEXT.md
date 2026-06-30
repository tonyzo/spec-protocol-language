# SPEC Protocol — 统一语言 (Ubiquitous Language)

> 本文件是 SPEC Protocol 项目的共享术语表。所有代码、文档、对话必须使用此处定义的语言。

## 职责边界（核心共识）

SPEC Protocol 是一个 **compliance 合规引擎**，负责对工程参数做规范规则匹配与工艺判定。

**不做数值计算** —— 厚度/应力/外压/迭代/查图等计算由外部计算软件完成，将来通过 MCP 或计算书接入。
**限值由外部提供** —— SPEC 不查材料库、不查图、不计算限值，只做「参数 vs 限值」的匹配判定。

```
              ┌─ parameters: {δ:16mm, σθ:120MPa} ─┐
计算软件 ────┤                                    ├──► SPEC compliance 引擎 ──RVM合规报告──► PDMS
              └─ limits: {δ_min:12mm, [σ]^t:163MPa}┘   (规则匹配+量纲校验)         (AVEVA)
```

- 参数表**分区组织**：`parameters`（参数值）与 `limits`（限值）分两个区，规则断言跨区引用。
- 报告输出为 **RVM 协作格式**（对接 AVEVA PDMS/E3D 工厂设计软件），非建筑 BIM 的 BCF。
- **量纲校验保留**：参数携带 `unit`，断言比对前先校验单位一致性（mm 与 m 混用报错）。

## 核心术语

### Parameter（参数）
计算软件输出的一个可验证值。包含：名称、值、单位、来源（可选）。
- 例：壁厚 `δ = 16 mm`；环向应力 `σθ = 120 MPa`。

### Limit（限值）
判据的边界值，由计算软件提供（SPEC 不计算）。
- 例：最小厚度 `δ_min = 12 mm`；许用应力 `[σ]^t = 163 MPa`。

### SpecRule（规范规则）
GB150 中一条可机器判定的合规要求。每条规则包含：
- **适用条件 (applicability)**：决定规则是否生效的前提
- **判据 (assertion)**：参数与限值的关系约束
- **严重度 (severity)**：`error` / `warning` / `info`
- **条款溯源 (clause)**：GB150 条款号 + 公式号

### Assertion（断言）
参数与限值的关系约束表达式。支持：
- 比较运算：`>=`、`<=`、`==`、`!=`、`>`、`<`
- 布尔组合：`AND`、`OR`、`NOT`
- 条件触发：`when <条件> then <判据>`

### Applicability（适用条件）
决定一条规则是否对给定参数集生效的前提。
- 例：当 `元件类型 = 内压圆筒` 且 `设计温度 > -20℃` 时，规则 `GB150.3-5.3` 生效。

### ComplianceReport（合规报告）
规则匹配的输出，含每条规则的 `pass` / `fail` 状态、违反条款号、溯源路径。
输出格式为 **RVM 协作格式**，对接 AVEVA PDMS/E3D 工厂设计软件（非建筑 BIM 的 BCF）。

### Violation（违规）
一条未满足的断言。含：规则 ID、条款号、实际值、限值、偏差量。

### ComplianceEngine（合规引擎）
SPEC 的核心组件。接收参数表 + 规则集，执行匹配判定，输出合规报告。**不包含任何数值计算能力。**

### UnitCheck（量纲校验）
断言求值前的预处理步骤。校验参数与限值的 `unit` 是否一致（如同为 mm），不一致则报错，不静默换算。
- **数值字面量例外**：规则中直接写的数值（如 `38`、`0.25`）unit 为空，视为"无单位约束"，不参与校验。
- 仅当两侧 unit 均非空且不一致时才报 `UnitMismatch`。

### RuleLibrary（规则库）
所有标准的全部 SpecRule 集合，按标准号分目录存储。
- 路径: `rules/library/{standard}/{chapter}-{topic}.yaml`
- 规则库中的规则是"候选规则"，尚未绑定到任何设计类型。
- 未来覆盖一个专业 100+ 项标准。

### ReviewPoint（审查点）
一条经人工确认、对特定设计类型生效的 SpecRule。
- 粒度: 1 审查点 = 1 SpecRule。
- 状态: `Candidate`（候选）→ `Confirmed`（已确认）→ `Sedimented`（已沉淀）。

### ReviewPointSet（审查点集合）
一个设计类型的全部已确认审查点，存储为 YAML 文件。
- 路径: `rules/confirmed/{design_type}/{standard}-{clause}-{year}.yaml`
- 文件名含标准号与年份，标准更新时替换文件。

### ReviewPointRegistry（审查点注册表）
索引文件，映射设计类型 → 审查点文件列表。
- 路径: `rules/registry.json`
- 记录每个文件的 `confirmed_at` 时间戳。

### DiscoveryEngine（发现引擎）
管理审查点生命周期的组件。执行发现→确认→沉淀→加载四步流程。
- `discover`: 从规则库中按 applicability 筛选候选审查点。
- `confirm`: 人工确认/拒绝每条候选。
- `sediment`: 将已确认规则保存为 YAML 并更新注册表。
- `load_confirmed`: 加载已沉淀的审查点集合。

## 工艺合规判据形态（已确认全部支持）

1. **存在性检查**：如「是否进行了 100% RT」「是否有 PQR 记录」——布尔存在性。
2. **参数范围比对**：如「预热温度 ≥ 150℃」「保压时间 ≥ 30min」——参数 vs 阈值。
3. **组合条件逻辑**：如「检测比例 ≥ 20% 且 合格级别 ≥ II 级」——AND/OR 组合。
4. **条件触发规则**：如「当 δ > 38mm 时必须进行焊后热处理」——when ... then ... 。

## 审查点生命周期（ADR-006）

```
规则库 (RuleLibrary)
    │
    │ 1. discover: 按 ParameterTable.applicability 自动筛选
    ▼
候选审查点 (CandidateReviewPoint[])
    │
    │ 2. confirm: 人工逐条确认/拒绝
    ▼
已确认审查点 (SpecRule[])
    │
    │ 3. sediment: 保存为 YAML + 更新注册表
    ▼
审查点集合 (ReviewPointSet)
    │
    │ 4. load: 后续直接加载，无需再确认
    ▼
ComplianceEngine.check()
```

## 边界声明（NOT 列表）

- SPEC **不做**：表达式求值、迭代求解、对数查图插值、材料库查询、限值计算、几何拓扑处理。
- 这些是外部计算软件的职责，SPEC 只消费其结果。
- SPEC **保留**：量纲校验（unit 一致性检查）、规则匹配、工艺判定、合规报告生成、审查点生命周期管理。

## 架构总览

### 代码结构

```
spec_protocol/
├── Cargo.toml                    # workspace 根配置
├── CONTEXT.md                    # 本文件：统一语言
├── docs/adr/                     # 架构决策记录
│   ├── ADR-001-职责边界-只做compliance.md
│   ├── ADR-002-核心语言-Rust加TS-SDK.md
│   ├── ADR-003-输入契约-结构化参数表.md
│   ├── ADR-004-报告格式-RVM协作格式对接PDMS.md
│   ├── ADR-005-量纲校验与参数表分区.md
│   └── ADR-006-审查点生命周期管理.md
├── crates/
│   └── spec_core/
│       ├── Cargo.toml            # crate 依赖
│       └── src/
│           ├── lib.rs            # 模块入口 + re-export
│           ├── model.rs          # 数据模型 (Quantity/ParameterTable/ComplianceReport)
│           ├── rule.rs           # 规则定义 (SpecRule/Assertion/Operand)
│           ├── assertion.rs      # 断言求值器 (比较+布尔运算+量纲校验)
│           ├── engine.rs         # 合规引擎 (ComplianceEngine)
│           ├── registry.rs       # 规则库 + 审查点注册表 (RuleLibrary/ReviewPointRegistry)
│           ├── discovery.rs      # 审查点发现引擎 (DiscoveryEngine)
│           ├── report.rs         # 报告适配器 (ReportWriter/JsonWriter/RvmWriter)
│           └── bin/
│               └── spec_cli.rs   # CLI 命令行工具
├── rules/                        # 规则库 + 审查点集合
│   ├── library/                  # 规则库：所有标准的全部规则
│   │   └── GB150.3/
│   │       └── 5.3-内压圆筒.yaml
│   ├── confirmed/                # 已确认的审查点集合（按设计类型）
│   │   └── 内压圆筒/
│   │       └── GB150.3-5.3-2024.yaml
│   └── registry.json             # 审查点注册表
└── input/                        # 输入参数表示例 (JSON)
    └── params.json
```

### 核心数据流

**常规审查（已有审查点集合）**:
```
rules/confirmed/{type}/   input/*.json
     │                         │
     │ SpecRule[] (已沉淀)      │ ParameterTable + DesignContext
     │                         │
     └────────┬────────────────┘
              ▼
     ComplianceEngine
     ├─ 1. 适用性筛选 (eval applicability)
     ├─ 2. 断言求值 (eval assertion + 量纲校验)
     └─ 3. 生成 RuleResult / Violation
              │
              ▼
     ComplianceReport → JsonWriter / RvmWriter
```

**首次审查（无审查点集合）**:
```
rules/library/            input/*.json
     │                         │
     │ 全部 SpecRule           │ ParameterTable (含 attrs.element_type)
     │                         │
     └────────┬────────────────┘
              ▼
     DiscoveryEngine.discover()
     ├─ 1. 遍历规则库全部规则
     ├─ 2. 逐条评估 applicability
     └─ 3. 输出 CandidateReviewPoint[]
              │
              ▼
     人工确认 (confirm)
              │
              ▼
     DiscoveryEngine.sediment()
     ├─ 保存为 rules/confirmed/{type}/{std}-{clause}-{year}.yaml
     └─ 更新 registry.json
```

### Assertion 类型映射

| 判据形态 | Assertion 变体 | YAML 写法 |
|---------|---------------|----------|
| 存在性检查 | `Exists(Operand)` | `exists: {attr: pwht_done}` |
| 参数范围比对 | `Ge/Le/Gt/Lt/Eq/Ne` | `le: [{param: sigma}, {limit: sigma_allow}]` |
| 组合条件逻辑 | `And/Or/Not` | `and: [ge: [...], le: [...]]` |
| 条件触发 | `When { cond, then }` | `when: { cond: {...}, then: {...} }` |

### CLI 用法

```bash
# 常规审查（已有审查点集合）
spec_cli check --design-type 内压圆筒 --params input/params.json --format json

# 首次审查：发现候选审查点
spec_cli discover --params input/params.json --output candidates.json

# 确认并沉淀审查点集合
spec_cli confirm --input candidates.json --design-type 内压圆筒
```

## 术语整理流程 (ADR-007)

### TermExtractor (术语提取工具)

独立工具脚本,负责将标准原文(PDF/Word/Markdown)整理为SPEC Protocol可用的术语表和参数映射。

**触发时机**:
- 新标准接入(首次)
- 标准更新(换版)
- 定期批量整理(全量校验)

**输入**: 标准原文(PDF/Word/Markdown)

**输出**:
1. `rules/library/{标准号}/param_mapping.json` — 机器可读的参数映射表
2. `docs/{标准号}-术语定义与概念映射.md` — 人类可读的术语文档
3. `docs/{标准号}-条款引用关系.md` — 标准内部/跨标准引用图
4. `rules/library/{标准号}/*.yaml` — 规则YAML骨架

**核心数据结构**:
```json
{
  "terms": [
    {
      "term_id": "pressure.design",
      "standard_term": "设计压力",
      "spec_mapping": {
        "type": "parameter",      // parameter/limit/attr
        "name": "design_pressure",
        "unit": "MPa"
      },
      "source_clause": "3.1.4",
      "confidence": "high"        // high/medium/low
    }
  ]
}
```

**参数分类原则**:
- `parameter` — 计算软件提供的计算值(应力/厚度/压力等)
- `limit` — 标准规定的限值/阈值(最小壁厚/许用应力等)
- `attr` — 分类属性,用于适用性筛选(元件类型/材料类别等)

### 整理流程
```
标准原文 → 术语提取 → 参数映射 → 文档生成 → 规则骨架 → 校验一致性
```

详见: [标准术语整理指南](docs/标准术语整理指南.md)

### 后续路线

- [x] 审查点生命周期管理 (ADR-006)
- [x] 术语提取与参数映射工具 (ADR-007)
- [ ] 术语提取工具实现 (term_extractor CLI)
- [ ] TS SDK：通过 napi-rs 包装 Rust 核心，生成 npm 包
- [ ] MCP 集成：将计算软件接口封装为 MCP server，自动填充 parameters/limits
- [ ] RVM schema：与 PDMS 侧对齐 RvmWriter 的输出格式
- [ ] 规则库扩展：按 GB150.3 章节逐步编写更多 SpecRule YAML，最终覆盖 100+ 标准
- [ ] 审查点集合增量更新：标准更新时仅更新变更的审查点文件
