# RVM 协作格式 Schema

> ADR-004: ComplianceReport 输出为 RVM 协作格式，对接 AVEVA PDMS/E3D。
> 本文档定义字段映射规范，待与 PDMS 侧确认后调整。

## 输出结构

RVM 报告分三个区域：

### 1. 头部元数据（注释行）

```
# RVM Compliance Report
# Generated: 2024-06-29T10:30:00
# Standard: GB/T 150.3-2024
# Element: V-101.SHELL (内压圆筒)
# Summary: 5 rules checked, 4 passed, 1 failed (1 errors, 0 warnings)
```

| 字段 | 来源 | 说明 |
|------|------|------|
| Generated | 系统时间 | ISO 8601 时间戳 |
| Standard | DesignContext.standard | 标准号+年份 |
| Element | DesignContext.element_id + element_type | 设备位号+类型 |
| Summary | ReportSummary | 规则总数、通过/失败数 |

### 2. 规则结果区（人类可读）

每条规则一行，标注状态：

```
# [PASS] GB150.3-5.3.sigma (clause 5.3)
# [FAIL(ERROR)] GB150.3-5.3.delta_min (clause 5.3)
#   delta >= delta_min: 10.0 vs 12.0 (deviation: -2.0) [mm]
# [N/A] GB150.3-5.3.phi_range (clause 表 2)
# [SKIP] GB150.4-7.PWHT (clause 7.3)
```

状态码：
- `PASS` — 规则通过
- `FAIL(ERROR)` — 规则未满足，严重度 Error
- `FAIL(WARN)` — 规则未满足，严重度 Warning
- `FAIL(INFO)` — 规则未满足，严重度 Info
- `N/A` — 规则不适用（applicability 未命中）
- `SKIP` — 缺少参数，无法判定

### 3. 违规详情区（PDMS 属性格式）

每条违规以 tab 分隔的 key-value 对输出，可直接导入 PDMS 属性表：

```
RULE_ID     GB150.3-5.3.delta_min
CLAUSE      5.3
ASSERTION   delta >= delta_min
SEVERITY    ERROR
ACTUAL      10
EXPECTED    12
UNIT        mm
DEVIATION   -2
MESSAGE     壁厚 10.0 mm 小于最小允许壁厚 12.0 mm
```

| 属性名 | 类型 | 说明 |
|--------|------|------|
| RULE_ID | String | 规则 ID，如 GB150.3-5.3.sigma |
| CLAUSE | String | 标准条款号，如 5.3 |
| ASSERTION | String | 断言描述，如 delta >= delta_min |
| SEVERITY | Enum | ERROR / WARNING / INFO |
| ACTUAL | Float | 实际值 |
| EXPECTED | Float | 限值 |
| UNIT | String | 单位（mm, MPa 等） |
| DEVIATION | Float | 偏差量 = actual - expected |
| MESSAGE | String | 人类可读的违规描述 |

## PDMS 对接说明

当前格式为结构化文本，可通过以下方式导入 PDMS：
1. **属性表导入** — 解析 tab 分隔的 key-value 对，写入 PDMS 属性
2. **宏脚本** — 生成 PDMS 宏命令，自动设置属性
3. **数据库直写** — 通过 PDMS DB API 直接写入

待与 PDMS 侧确认后，可调整输出格式为 XML/JSON/宏脚本。
