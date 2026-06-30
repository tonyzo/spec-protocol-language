# ADR-002: 核心实现语言 —— Rust 核心 + TypeScript SDK

## 状态
已确认（2026-06-29）

## 决策
- **Rust**：实现核心 compliance 引擎（规则加载、匹配判定、报告生成、serde 序列化）。
- **TypeScript**：SDK 层，对接前端 UI / CAD 插件 / VS Code。
- **通信**：JSON-RPC 2.0 over stdio / TCP。

## 理由
- Rust 的强类型 + serde 天然适合规则模型的序列化/反序列化，且规则匹配是高吞吐判定场景。
- TS SDK 对接工程前端生态，降低集成门槛。
- 既然 SPEC 不做数值计算（见 ADR-001），Rust 的性能优势主要服务于规则匹配吞吐，而非数学运算。

## 后果
- 需要 Rust ↔ TS 的 FFI / IPC 桥接层。
- 规则文件（SP-YAML）在 Rust 端解析，TS 端仅消费结构化结果。
