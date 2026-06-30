# Matt Pocock Skills 中英文对照表

> 所有 skills 已安装在个人目录：`%USERPROFILE%\.agents\skills\`

## 📋 Skills 分类对照

### 🎯 核心工程 Skills (Engineering)

| 英文命令 | 中文名称 | 用途说明 | 使用场景 |
|---------|---------|---------|---------|
| `/ask-matt` | 智能路由 | 帮您选择最合适的 skill | 不确定用哪个 skill 时使用 |
| `/improve-codebase-architecture` | 改进代码库架构 | 扫描代码库，找出架构改进机会 | 定期审查代码库架构质量 |
| `/tdd` | 测试驱动开发 | 红-绿-重构循环开发 | 开发新功能或修复 bug |
| `/diagnosing-bugs` | 调试诊断 | 系统化诊断疑难 bug | 遇到难以调试的问题 |
| `/codebase-design` | 代码库设计 | 深度模块设计原则 | 设计或改进模块接口 |
| `/domain-modeling` | 领域建模 | 构建和锐化项目领域模型 | 定义领域术语和概念 |
| `/prototype` | 快速原型 | 构建一次性原型验证设计 | 验证设计方案是否可行 |
| `/to-prd` | 生成 PRD | 将对话转化为产品需求文档 | 需求讨论后生成文档 |
| `/to-issues` | 拆分任务 | 将计划拆分为可执行任务 | 准备开始开发任务 |

### 📝 需求澄清 Skills (Grilling)

| 英文命令 | 中文名称 | 用途说明 | 使用场景 |
|---------|---------|---------|---------|
| `/grill-me` | 需求访谈 | 严格访谈直到决策树完整 | 开始新功能前澄清需求 |
| `/grill-with-docs` | 需求访谈+文档 | 访谈同时生成 ADR 和术语表 | 需求澄清并建立项目文档 |
| `/grilling` | 访谈引擎 | 底层访谈循环 | 被其他 grill skill 调用 |

### 🔧 项目管理 Skills (Project Management)

| 英文命令 | 中文名称 | 用途说明 | 使用场景 |
|---------|---------|---------|---------|
| `/triage` | 问题分类 | 管理问题状态机 | 处理 GitHub/Linear issues |
| `/handoff` | 交接文档 | 压缩对话为交接文档 | 切换 agent 或会话 |
| `/teach` | 教学助手 | 多会话教学新概念 | 学习新技能或概念 |

### ✍️ 写作 Skills (Writing)

| 英文命令 | 中文名称 | 用途说明 | 使用场景 |
|---------|---------|---------|---------|
| `/writing-great-skills` | 编写优秀 Skills | Skill 编写参考指南 | 创建或编辑 skills |
| `/writing-beats` | 写作节拍 | 组织材料为连贯旅程 | 写文章或教程 |
| `/writing-fragments` | 写作碎片 | 挖掘零散想法 | 头脑风暴阶段 |
| `/writing-shape` | 写作成型 | 逐段塑造文章 | 文章精修阶段 |
| `/edit-article` | 编辑文章 | 改进文章结构和清晰度 | 修订文章草稿 |

### 🛠️ 工具 Skills (Tools)

| 英文命令 | 中文名称 | 用途说明 | 使用场景 |
|---------|---------|---------|---------|
| `/setup-matt-pocock-skills` | 配置 Skills | 配置问题跟踪器和标签 | 首次使用前配置 |
| `/setup-pre-commit` | 配置预提交 | 设置 Husky 预提交钩子 | 添加代码提交检查 |
| `/git-guardrails-claude-code` | Git 安全护栏 | 阻止危险 git 命令 | 防止误操作 |
| `/resolving-merge-conflicts` | 解决合并冲突 | 处理 git 合并冲突 | 遇到合并冲突时 |
| `/migrate-to-shoehorn` | 迁移到 Shoehorn | 替换 `as` 类型断言 | 测试代码重构 |
| `/scaffold-exercises` | 脚手架练习 | 创建练习目录结构 | 课程材料准备 |
| `/obsidian-vault` | Obsidian 笔记 | 管理 Obsidian 笔记 | 知识管理 |

### 🎨 设计 Skills (Design)

| 英文命令 | 中文名称 | 用途说明 | 使用场景 |
|---------|---------|---------|---------|
| `/design-an-interface` | 设计接口 | 生成多个接口设计方案 | 探索 API 设计选项 |
| `/ubiquitous-language` | 统一语言 | 提取 DDD 风格术语表 | 建立领域术语 |
| `/decision-mapping` | 决策映射 | 将想法转化为调查票 | 规划调研方向 |
| `/request-refactor-plan` | 重构计划 | 创建详细重构计划 | 规划大规模重构 |

### 🔄 其他 Skills (Others)

| 英文命令 | 中文名称 | 用途说明 | 使用场景 |
|---------|---------|---------|---------|
| `/implement` | 实现功能 | 基于 PRD 实现功能 | 执行开发任务 |
| `/review` | 代码审查 | 审查代码变更 | PR 审查或代码回顾 |
| `/qa` | 质量保证 | 交互式 QA 会话 | 报告 bug 或问题 |
| `/loop-me` | 循环访谈 | 工作流规范访谈 | 定义工作流规格 |
| `/wizard` | 向导生成 | 生成交互式 bash 向导 | 手动流程自动化 |

---

## 🚀 推荐工作流

### 新功能开发流程
```
1. /grill-with-docs      → 澄清需求，生成文档
2. /to-prd              → 生成产品需求文档
3. /to-issues           → 拆分为可执行任务
4. /tdd                 → 测试驱动开发实现
5. /improve-codebase-architecture → 审查架构质量
```

### Bug 修复流程
```
1. /diagnosing-bugs     → 诊断问题根因
2. /tdd                 → 先写测试修复
3. /review              → 审查修复质量
```

### 代码库改进流程
```
1. /improve-codebase-architecture → 扫描改进机会
2. /design-an-interface → 设计新接口
3. /request-refactor-plan → 制定重构计划
4. /implement           → 实施重构
```

---

## 📌 备注

- 所有 skills 均为全局安装，可在任何项目中使用
- Skills 会随对话自动加载到 Qoder
- 输入 `/` 后跟 skill 名称即可使用
- 详细文档：https://github.com/mattpocock/skills
