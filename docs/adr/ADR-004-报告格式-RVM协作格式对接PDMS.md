# ADR-004: 报告格式 —— RVM 协作格式对接 AVEVA PDMS/E3D

## 状态
已确认（2026-06-29）

## 决策
ComplianceReport 的输出格式为 **RVM 协作格式**，对接 AVEVA PDMS/E3D 工厂设计软件。

- 不采用建筑 BIM 的 BCF 格式（本场景为化工厂设计，非建筑 BIM）。
- 报告输出层设计为**可插拔适配器**：RVM 适配器为首选实现，预留 JSON 适配器用于测试/调试。

## 背景
项目目录名为「RVM Model Checker」，实际工程环境为 AVEVA PDMS/E3D 工厂设计软件。
RVM 通常是 AVEVA Review 的三维模型导出格式。「RVM 协作格式」指能与 PDMS 属性/报告体系对接的结构化输出。

## 后果
- 需要进一步明确 RVM 协作格式的具体 schema（待用户提供样例或与 PDMS 侧对齐）。
- 报告模块抽象为 `ReportWriter` trait，RVM 实现为其中一个 adapter。
- 内部 ComplianceReport 先用中性数据结构表达，再由 adapter 转为目标格式。

## 相关
- ADR-001：职责边界
