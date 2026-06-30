# ASME Boiler and Pressure Vessel Code Section VIII Division 1 术语定义与概念映射表

> 本文档将ASME Boiler and Pressure Vessel Code Section VIII Division 1标准中的术语、概念与SPEC Protocol的参数/限值/属性建立映射关系。
> 
> **数据来源**: ASME BPVC Section VIII Division 1 - Rules for Construction of Pressure Vessels
> 
> **映射原则**: 
> - 标准术语 → SPEC参数名(`parameters`/`limits`/`attrs`)
> - 明确哪些参数需计算软件提供
> - 标注术语出处条款号

---

## 1. 核心术语定义表

### 1.1 压力相关术语

| 序号 | 标准术语 | 术语定义 | SPEC映射 | 数据类型 | 单位 | 来源条款 | 置信度 |
|-----|---------|---------|---------|---------|-----|---------|--------|
| 1 | Design Pressure | The pressure used in the design of a component, together with any coincident static or dynamic loads | design_pressure | f64 | psi | UG-21 | ✅ high |
| 2 | Maximum Allowable Working Pressure | The maximum gauge pressure permissible at the top of a vessel in its normal operating position | MAWP | f64 | psi | UG-98 | ✅ high |
| 3 | Test Pressure | The pressure to which a vessel is subjected during a hydrostatic or pneumatic test | test_pressure | f64 | psi | UG-99 | ✅ high |

### 1.2 温度相关术语

| 序号 | 标准术语 | 术语定义 | SPEC映射 | 数据类型 | 单位 | 来源条款 | 置信度 |
|-----|---------|---------|---------|---------|-----|---------|--------|
| 1 | Design Temperature | The temperature of the metal used in the design of a component | design_temperature | f64 | °F | UG-20 | ✅ high |
| 2 | Minimum Design Metal Temperature | The lowest temperature expected in service | MDMT | f64 | °F | UG-20 | ✅ high |

### 1.3 厚度相关术语 (关键!)

| 序号 | 标准术语 | 术语定义 | SPEC映射 | 数据类型 | 单位 | 来源条款 | 置信度 |
|-----|---------|---------|---------|---------|-----|---------|--------|
| 1 | Required Thickness | The minimum thickness of a component calculated by the design formulas | delta_calc | f64 | in | UG-27 | ✅ high |
| 2 | Nominal Thickness | The thickness to which the material is ordered, including corrosion allowance and mill tolerance | delta | f64 | in | UG-25 | ✅ high |
| 3 | Effective Thickness | The nominal thickness minus corrosion allowance and mill tolerance | delta_effective | f64 | in | UG-25 | ✅ high |

### 1.4 应力与材料性能术语

| 序号 | 标准术语 | 术语定义 | SPEC映射 | 数据类型 | 单位 | 来源条款 | 置信度 |
|-----|---------|---------|---------|---------|-----|---------|--------|
| 1 | Allowable Stress | The maximum stress permitted by the code for a given material at a given temperature | sigma_allow | f64 | psi | UG-23 | ✅ high |
| 2 | Calculated Stress | The stress in a component calculated by the design formulas | sigma | f64 | psi | UG-27 | ✅ high |

### 1.6 材料性能术语

| 序号 | 标准术语 | 术语定义 | SPEC映射 | 数据类型 | 单位 | 来源条款 | 置信度 |
|-----|---------|---------|---------|---------|-----|---------|--------|
| 1 | Material Type | The classification of material | material_category | string | - | UG-5 | ✅ high |
| 2 | Material Specification | The ASTM or ASME material specification | material_grade | string | - | UG-5 | ✅ high |

## 2. 分类属性术语(attrs)

| 属性名 | 可能值 | 说明 | 来源条款 |
|-------|-------|------|----------|
| element_type | Shell, Head, Flange, Nozzle, Saddle, Lug | The type of pressure vessel component | UG-1 |
| material_category | Carbon Steel, Low Alloy Steel, Stainless Steel, Aluminum, Copper, Nickel | The classification of material | UG-5 |
| material_grade | SA-516 Gr.70, SA-240 Type 304, SA-240 Type 316, SA-106 Gr.B | The ASTM or ASME material specification | UG-5 |
| ndt_ratio | - | The extent of non-destructive examination | UW-11 |
| test_type | Hydrostatic, Pneumatic, Combined Hydrostatic-Pneumatic | The type of pressure test | UG-99 |

## 3. 参数分组速查

### 3.1 设计参数
- design_pressure, design_temperature, MAWP, test_pressure

### 3.2 几何参数
- D_i, D_o, delta, delta_effective, corrosion_allowance

### 3.3 应力参数
- sigma, sigma_allow, sigma_hoop, sigma_longitudinal

### 3.4 限值参数
- delta_calc, MAWP, pressure_allow

### 3.5 材料性能
- R_m, R_eL, E_t

### 3.6 焊接与NDT
- phi, ndt_ratio, weld_joint_type

### 3.7 试验参数
- test_pressure_actual, hold_time

### 3.8 分类属性
- element_type, material_category, material_grade, test_type

## 4. 跨标准引用关系

### ASME_VIII→GB150
ASME VIII and GB150 have similar concepts but different units
**涉及术语**: design_pressure (psi vs MPa), design_temperature (°F vs °C), thickness (in vs mm), stress (psi vs MPa)

### ASME_VIII→ASME_II
ASME VIII references ASME Section II for material properties
**涉及术语**: sigma_allow, R_m, R_eL

## 5. 校验规则

- **All parameters must have unit field**: terms[*].spec_mapping.type=='parameter' implies spec_mapping.unit != null
- **All limits must have unit field**: terms[*].spec_mapping.type=='limit' implies spec_mapping.unit != null
- **Attrs cannot have unit**: terms[*].spec_mapping.type=='attr' implies spec_mapping.unit == null

---

**文档版本**: 1.0  
**创建日期**: 2026-06-30  
**数据来源**: ASME BPVC Section VIII Division 1 - Rules for Construction of Pressure Vessels  
**维护者**: SPEC Protocol 团队  
**状态**: 持续更新中
