# ADR-009: MCP 集成设计 — 计算软件接口封装

## 状态
设计中（2026-06-29）

## 背景

ADR-001 确定了 SPEC Protocol 只做 compliance 合规引擎，不做数值计算。数值计算由外部计算软件（如 SW6）完成。

当前的问题：
- 计算软件输出的参数表需要手动整理为 `input/params.json` 格式
- 没有标准化的接口来连接计算软件与 SPEC 合规引擎
- 限值（如许用应力）需要从标准表格中查找，当前依赖人工

CONTEXT.md 后续路线中提到"将计算软件接口封装为 MCP server，自动填充 parameters/limits"。

## 决策

设计 MCP（Model Context Protocol）server 接口规范，将计算软件封装为标准化的 MCP server。

### MCP Server 职责

```
计算软件 (SW6) ──MCP──► MCP Server ──► SPEC 合规引擎
                         │
                         ├─ tools/compute: 调用计算软件，返回参数+限值
                         ├─ tools/lookup_limit: 查询标准限值表
                         └─ resources/params: 输出结构化参数表
```

### 接口定义

#### Tool: `compute`

调用计算软件执行计算，返回参数表。

```json
{
  "method": "tools/call",
  "params": {
    "name": "compute",
    "arguments": {
      "standard": "GB150.3",
      "element_type": "内压圆筒",
      "inputs": {
        "design_pressure": { "value": 2.5, "unit": "MPa" },
        "design_temperature": { "value": 100, "unit": "℃" },
        "inner_diameter": { "value": 1000, "unit": "mm" }
      }
    }
  }
}
```

返回：
```json
{
  "parameter_table": {
    "parameters": {
      "delta": { "value": 16.0, "unit": "mm" },
      "sigma": { "value": 120.0, "unit": "MPa" }
    },
    "limits": {
      "delta_min": { "value": 12.0, "unit": "mm" },
      "sigma_allow": { "value": 163.0, "unit": "MPa" }
    },
    "attrs": {
      "element_type": "内压圆筒",
      "material_category": "碳钢"
    }
  }
}
```

#### Tool: `lookup_limit`

查询标准限值表（如许用应力、焊接系数）。

```json
{
  "method": "tools/call",
  "params": {
    "name": "lookup_limit",
    "arguments": {
      "standard": "GB150.2",
      "table": "许用应力",
      "material": "Q245R",
      "temperature": 100
    }
  }
}
```

返回：
```json
{
  "limit_name": "sigma_allow",
  "value": 163.0,
  "unit": "MPa",
  "source": "GB150.2 表1, Q245R, 100℃"
}
```

#### Resource: `params://current`

当前参数表（可直接被 SPEC 引擎消费）。

### 集成方式

1. **独立 MCP Server** — 作为独立进程运行，通过 stdio/TCP 与 SPEC CLI 通信
2. **CLI 集成** — `spec check --mcp-server /path/to/sw6-mcp` 自动调用 MCP server 获取参数
3. **批量集成** — `spec batch-extract` 中配置 MCP server，自动填充参数表

### 实现路线

- [ ] Phase 1: 定义 MCP server 接口规范（本文档）
- [ ] Phase 2: 实现 Mock MCP Server（返回模拟参数表）
- [ ] Phase 3: 实现 SW6 MCP Server（对接真实计算软件）
- [ ] Phase 4: 在 spec_cli 中集成 MCP client

## 后果

### 正面
- 计算软件与 SPEC 引擎解耦，通过标准接口通信
- 限值查找自动化，减少人工错误
- 支持多种计算软件（SW6、SW6-PLUS、LVO 等）

### 负面
- 需要实现 MCP server 和 client
- 计算软件的输出格式可能不统一，需要适配层
- MCP 协议尚在发展中，接口可能变化

## 相关
- ADR-001: 职责边界（SPEC 不做数值计算）
- ADR-003: 输入契约（结构化参数表）
