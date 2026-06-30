# SPEC Protocol 架构关联关系

本文档描述 SPEC Protocol 系统的完整架构关联关系,包括模块关系、数据流和标准覆盖。

---

## 1. 系统架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                    SPEC Protocol 系统架构                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐      ┌──────────────┐      ┌──────────┐ │
│  │ 计算软件     │─────►│ SPEC 合规引擎│─────►│ PDMS/E3D │ │
│  │ (外部MCP)    │      │              │      │ (AVEVA)  │ │
│  └──────────────┘      └──────────────┘      └──────────┘ │
│         │                     │                            │
│         │ 结构化参数表        │ RVM合规报告                │
│         │ (parameters/        │ (JSON/RVM格式)             │
│         │  limits/attrs)      │                            │
│         ▼                     ▼                            │
│  ┌──────────────────────────────────────────────────┐     │
│  │              规则库管理系统                       │     │
│  │  ┌─────────────┐  ┌────────────────────────┐    │     │
│  │  │ RuleLibrary │  │ ReviewPointRegistry    │    │     │
│  │  │ (规则库)    │  │ (审查点注册表)         │    │     │
│  │  └─────────────┘  └────────────────────────┘    │     │
│  │         │                    │                   │     │
│  │         ▼                    ▼                   │     │
│  │  ┌─────────────┐  ┌────────────────────────┐    │     │
│  │  │ Discovery   │  │ Sedimented Rules       │    │     │
│  │  │ Engine      │  │ (已沉淀审查点)         │    │     │
│  │  └─────────────┘  └────────────────────────┘    │     │
│  └──────────────────────────────────────────────────┘     │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. 核心模块关联关系

### 2.1 模块依赖图

```
spec_core (核心库)
├── model.rs          # 数据模型层
│   ├── Quantity          # 带单位的数值
│   ├── ParameterTable    # 参数表(三分区)
│   ├── SpecRule          # 规则定义
│   ├── Assertion         # 断言枚举(4种判据)
│   ├── ComplianceReport  # 合规报告
│   └── ReviewPoint*      # 审查点生命周期数据结构
│
├── rule.rs           # 规则定义层
│   ├── Operand           # 操作数(参数/限值/属性/数值)
│   ├── Assertion         # 断言(Ge/Le/Gt/Lt/Eq/Ne/And/Or/Not/Exists/When)
│   └── SpecRule          # 完整规则结构
│
├── assertion.rs      # 断言求值层
│   ├── eval()            # 主求值函数
│   ├── compare()         # 比较运算(带量纲校验)
│   └── SpecError         # 错误类型
│
├── engine.rs         # 合规引擎层
│   ├── ComplianceEngine  # 引擎主结构
│   ├── from_yaml()       # YAML加载
│   └── check()           # 合规校核主流程
│
├── report.rs         # 报告输出层
│   ├── ReportWriter      # 输出适配器trait
│   ├── JsonWriter        # JSON格式输出
│   └── RvmWriter         # RVM格式输出(PDMS对接)
│
├── registry.rs       # 规则库管理层
│   ├── RuleLibrary       # 规则库加载
│   └── ReviewPointRegistry # 审查点注册表
│
└── discovery.rs      # 审查点生命周期层
    ├── DiscoveryEngine   # 发现引擎
    ├── discover()        # 发现候选审查点
    ├── confirm()         # 人工确认
    ├── sediment()        # 沉淀为文件
    └── load_set()        # 加载已沉淀集合
```

### 2.2 数据流关联

#### 常规审查流程(已有审查点集合)

```
用户输入                          规则加载                         合规校核
┌──────────────┐          ┌──────────────────┐          ┌──────────────────┐
│ params.json  │          │ confirmed/       │          │ ComplianceEngine │
│              │          │  └─内压圆筒/      │          │                  │
│ - context    │─────────►│     GB150.3-*.yaml│─────────►│ check()          │
│ - parameters │          │     GB150.4-*.yaml│          │                  │
│ - limits     │          └──────────────────┘          └────────┬─────────┘
│ - attrs      │                                                 │
└──────────────┘                                                 │
                                                                 ▼
                                                        ┌──────────────────┐
                                                        │ ComplianceReport │
                                                        │                  │
                                                        │ - results[]      │
                                                        │ - summary        │
                                                        └────────┬─────────┘
                                                                 │
                                    ┌────────────────────────────┼────────────────────┐
                                    ▼                            ▼                    ▼
                              ┌──────────┐                ┌──────────┐        ┌──────────────┐
                              │JsonWriter│                │RvmWriter │        │ 终端输出     │
                              └──────────┘                └──────────┘        └──────────────┘
```

#### 首次审查流程(无审查点集合)

```
用户输入                          规则发现                         人工确认
┌──────────────┐          ┌──────────────────┐          ┌──────────────────┐
│ params.json  │          │ library/         │          │ 审查人员         │
│              │          │  └─GB150.1/      │          │                  │
│ - attrs      │─────────►│  └─GB150.2/      │─────────►│ 1. 查看候选列表  │
│   (element_  │          │  └─GB150.3/      │          │ 2. 逐条确认/拒绝│
│    type)     │          │  └─GB150.4/      │          │ 3. 标记confirmed │
└──────────────┘          └────────┬─────────┘          └────────┬─────────┘
                                   │                             │
                                   ▼                             ▼
                          ┌──────────────────┐          ┌──────────────────┐
                          │ DiscoveryEngine  │          │ 沉淀文件         │
                          │                  │          │ confirmed/       │
                          │ discover()       │          │  └─内压圆筒/     │
                          │                  │          │     *.yaml       │
                          │ 返回候选审查点   │          │     registry.json│
                          └──────────────────┘          └──────────────────┘
```

---

## 3. GB150 标准覆盖关系

### 3.1 规则库目录结构

```
rules/
├── library/                      # 规则库(所有标准的全部候选规则)
│   ├── GB150.1/                  # 第1部分: 通用要求 (4条规则)
│   │   └── 4-通用要求.yaml
│   │       ├── GB150.1-1.2.pressure_range       (压力范围)
│   │       ├── GB150.1-1.3.temperature_range    (温度范围)
│   │       ├── GB150.1-1.4.min_diameter         (最小内径)
│   │       └── GB150.1-3.15.low_temp_vessel     (低温容器判定)
│   │
│   ├── GB150.2/                  # 第2部分: 材料 (6条规则)
│   │   └── 4-材料总体要求.yaml
│   │       ├── GB150.2-4.9.temp_upper_limit     (温度上限)
│   │       ├── GB150.2-4.10.temp_lower_limit    (温度下限)
│   │       ├── GB150.2-4.9.graphitization_warning (石墨化警示)
│   │       ├── GB150.2-4.9.austenitic_carbon    (含碳量要求)
│   │       ├── GB150.2-4.11.impact_toughness    (冲击韧性)
│   │       └── GB150.2-4.13.phase_ratio         (相比例)
│   │
│   ├── GB150.3/                  # 第3部分: 设计 (20条规则)
│   │   ├── 5.3-内压圆筒.yaml
│   │   │   ├── GB150.3-5.3.sigma                (应力校核)
│   │   │   ├── GB150.3-5.3.delta_min            (最小壁厚)
│   │   │   └── GB150.3-5.3.phi_range            (焊接系数)
│   │   ├── 5.4-球壳.yaml
│   │   │   ├── GB150.3-5.4.sigma_sphere         (球壳应力)
│   │   │   ├── GB150.3-5.4.delta_min_sphere     (球壳壁厚)
│   │   │   └── GB150.3-5.4.k_ratio_limit        (径比限制)
│   │   ├── 6-外压圆筒和球壳.yaml
│   │   │   ├── GB150.3-6.3.stability_cylinder   (外压稳定性)
│   │   │   ├── GB150.3-6.4.stability_sphere     (球壳稳定性)
│   │   │   ├── GB150.3-6.5.stiffener_inertia    (加强圈惯性矩)
│   │   │   └── GB150.3-6.3.calc_length_valid    (计算长度)
│   │   ├── 7-封头.yaml
│   │   │   ├── GB150.3-7.3.elliptical_stress    (椭圆封头应力)
│   │   │   ├── GB150.3-7.4.toroidal_stress      (碟形封头应力)
│   │   │   ├── GB150.3-7.3.elliptical_buckling  (屈曲判据)
│   │   │   ├── GB150.3-7.5.cone_angle_limit     (半顶角限制)
│   │   │   └── GB150.3-7.6.cone_reinforcement_area (加强面积)
│   │   ├── 8-开孔补强.yaml
│   │   │   ├── GB150.3-8.1.hole_diameter_limit  (开孔直径)
│   │   │   ├── GB150.3-8.5.reinforcement_area   (补强面积)
│   │   │   ├── GB150.3-8.4.hole_spacing         (开孔间距)
│   │   │   └── GB150.3-8.6.reinforce_plate_thickness (补强圈厚度)
│   │   └── 9-法兰.yaml
│   │       ├── GB150.3-9.5.flange_axial_stress  (轴向应力)
│   │       ├── GB150.3-9.5.flange_radial_stress (径向应力)
│   │       ├── GB150.3-9.5.flange_shear_stress  (剪应力)
│   │       ├── GB150.3-9.4.bolt_load            (螺栓载荷)
│   │       └── GB150.3-9.2.gasket_width         (垫片宽度)
│   │
│   └── GB150.4/                  # 第4部分: 制造检验 (17条规则)
│       ├── 7-热处理.yaml
│       │   └── GB150.4-7.PWHT_carbon            (PWHT触发)
│       ├── 7-焊接.yaml
│       │   ├── GB150.4-7.1.humidity_limit       (湿度限制)
│       │   ├── GB150.4-7.1.welding_temp_limit   (温度限制)
│       │   ├── GB150.4-7.2.preheat_temperature  (预热温度)
│       │   ├── GB150.4-7.3.pwht_thickness_trigger (PWHT厚度)
│       │   └── GB150.4-7.5.repair_count_limit   (返修次数)
│       ├── 9-无损检测.yaml
│       │   └── GB150.4-9.NDT_ratio              (检测比例)
│       ├── 10-无损检测.yaml
│       │   ├── GB150.4-10.1.full_ndt_trigger    (全部检测触发)
│       │   ├── GB150.4-10.2.partial_ndt_min_ratio (局部检测比例)
│       │   ├── GB150.4-10.3.rt_quality_level    (RT合格级别)
│       │   ├── GB150.4-10.3.ut_quality_level    (UT合格级别)
│       │   └── GB150.4-10.4.surface_ndt_trigger (表面检测触发)
│       └── 11-耐压试验.yaml
│           ├── GB150.4-11.3.hydro_test_pressure (液压试验压力)
│           ├── GB150.4-11.4.pneumatic_test_pressure (气压试验压力)
│           ├── GB150.4-11.2.test_temp_limit     (试验温度)
│           ├── GB150.4-11.4.hold_time_min       (保压时间)
│           └── GB150.4-11.5.leak_test_trigger   (泄漏试验触发)
│
├── confirmed/                    # 已确认的审查点集合
│   └── {设计类型}/
│       ├── GB150.3-5.3-2024.yaml
│       ├── GB150.4-7.3-2024.yaml
│       └── ...
│
└── registry.json                 # 审查点注册表
```

### 3.2 标准条款覆盖率

| 标准部分 | 章节数 | 已生成规则数 | 覆盖条款 | 覆盖率(估算) |
|---------|-------|------------|---------|------------|
| GB150.1 (通用) | 4章 | 4 | 1.2, 1.3, 1.4, 3.1.15 | ~15% |
| GB150.2 (材料) | 9章 | 6 | 4.9, 4.10, 4.11, 4.13 | ~20% |
| GB150.3 (设计) | 9章+附录 | 20 | 5.3, 5.4, 6.3-6.5, 7.3-7.6, 8.1-8.6, 9.2-9.5 | ~35% |
| GB150.4 (制造) | 13章+附录 | 17 | 7.1-7.5, 9.2, 10.1-10.4, 11.2-11.5 | ~25% |
| **总计** | **35章** | **47条** | **核心条款** | **~25%** |

---

## 4. 审查点生命周期关联

### 4.1 状态转换图

```
┌─────────────┐
│ Candidate   │ 候选审查点
│ (规则库中)  │
└──────┬──────┘
       │ discover() 自动筛选
       ▼
┌─────────────┐
│ Candidate   │ 待确认审查点
│ ReviewPoint │ (人工审查界面)
└──────┬──────┘
       │ confirm() 人工确认
       ├───────────────┐
       ▼               ▼
  confirmed: true   confirmed: false
       │               │
       ▼               ▼
┌─────────────┐  ┌─────────────┐
│ Sedimented  │  │ Rejected    │ 拒绝的规则
│ (沉淀为YAML)│  │ (不保存)    │
└──────┬──────┘  └─────────────┘
       │
       ▼
┌─────────────┐
│ Confirmed   │ 已确认审查点
│ ReviewPoint │ (可重复加载)
└──────┬──────┘
       │ load_set() 后续审查
       ▼
┌─────────────┐
│ Compliance  │ 合规校核执行
│ Engine      │ (生成报告)
└─────────────┘
```

### 4.2 文件关联关系

```
设计类型 (如"内压圆筒")
    │
    ├─► rules/confirmed/内压圆筒/
    │      ├─ GB150.3-5.3-2024.yaml  (应力校核、壁厚校核、焊接系数)
    │      ├─ GB150.4-7.3-2024.yaml  (PWHT、焊接)
    │      └─ GB150.4-10-2024.yaml   (无损检测)
    │
    └─► rules/registry.json
           └─ "内压圆筒": [
                { standard: "GB150.3", clause: "5.3", file_path: "...", rule_count: 3 },
                { standard: "GB150.4", clause: "7.3", file_path: "...", rule_count: 2 },
                ...
              ]
```

---

## 5. 数据模型关联

### 5.1 参数表三分区

```json
{
  "parameters": {
    // 计算软件提供的计算值
    "sigma": { "value": 120.0, "unit": "MPa" },
    "delta": { "value": 16.0, "unit": "mm" },
    "design_pressure": { "value": 2.5, "unit": "MPa" }
  },
  "limits": {
    // 计算软件提供的限值/阈值
    "sigma_allow": { "value": 163.0, "unit": "MPa" },
    "delta_min": { "value": 12.0, "unit": "mm" }
  },
  "attrs": {
    // 分类属性(字符串,用于适用性筛选)
    "element_type": "内压圆筒",
    "material_category": "碳钢",
    "pwht_done": "true"
  }
}
```

### 5.2 规则结构映射

```yaml
# YAML 规则文件
- id: GB150.3-5.3.sigma
  clause: "5.3 / 公式 5-5"
  severity: error
  applicability:
    type: eq
    value: [{attr: element_type}, "内压圆筒"]
  assertion:
    type: le
    value: [{param: sigma}, {limit: sigma_allow}]
  message: "环向应力 {actual} MPa 超过许用应力 {expected} MPa"

# 映射到 Rust 数据结构
SpecRule {
  id: "GB150.3-5.3.sigma",
  clause: "5.3 / 公式 5-5",
  severity: Severity::Error,
  applicability: Some(Assertion::Eq(
    Operand::AttrRef { attr: "element_type" },
    Operand::Str("内压圆筒".into())
  )),
  assertion: Assertion::Le(
    Operand::ParamRef { param: "sigma" },
    Operand::LimitRef { limit: "sigma_allow" }
  ),
  message: Some("环向应力 {actual} MPa 超过许用应力 {expected} MPa".into())
}
```

---

## 6. 扩展路径

### 6.1 新标准接入流程

```
1. 创建 rules/library/{新标准号}/ 目录
2. 按章节生成 YAML 规则文件
3. 规则 ID 格式: {标准号}-{条款号}.{描述}
4. DiscoveryEngine 自动发现新标准规则
5. 用户确认后沉淀到 confirmed/
```

### 6.2 未来扩展方向

- [ ] 规则版本管理(标准换版自动提醒)
- [ ] 规则依赖关系(某些规则需前置规则通过)
- [ ] 规则优先级(冲突规则处理策略)
- [ ] 规则测试用例(每条规则附带测试参数)
- [ ] 规则可视化(图形化规则编辑器)
- [ ] MCP 集成(计算软件自动提供 parameters/limits)

---

## 7. 关键设计决策索引

| 决策 | ADR 编号 | 说明 |
|-----|---------|------|
| 只做compliance,不做数值计算 | ADR-001 | SPEC职责边界 |
| Rust核心 + TS SDK | ADR-002 | 实现语言选择 |
| 输入=结构化参数表 | ADR-003 | 数据契约 |
| 输出=RVM格式 | ADR-004 | 报告格式 |
| 量纲校验+参数表分区 | ADR-005 | 单位处理 |
| 审查点生命周期管理 | ADR-006 | 规则沉淀机制 |

---

**文档版本**: 1.0  
**更新日期**: 2025-06-29  
**维护者**: SPEC Protocol 团队
