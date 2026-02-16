# Phase 2 Vibe Coding 使用指南

## 使用方式

### 第一步: Stage A (探索性架构设计)

1. 开一个新的 agent 会话
2. 将以下两个文件作为附件提供给 agent:
   - `prompt/PHASE2-STAGE-A.md` (架构愿景, agent 的设计北极星)
   - `prompt/PHASE2_KICK_A.md` (执行指令, 直接复制 code fence 里的文本作为 prompt 发送)
3. Agent 会: 读取现有 `spec/phase2/` 草稿 → 做选项分析 → 更新 spec 文件 → 在每个文件末尾标注 "Freeze Candidates"
4. **你在这里做人工审查**: 检查 agent 的设计方向, 选项取舍是否合理, 必要时给反馈让它修正

### 第二步: Stage B (合同冻结)

1. Stage A 审查通过后, **开一个新的 agent 会话** (干净的上下文)
2. 将以下两个文件作为附件提供:
   - `prompt/PHASE2-STAGE-B.md` (冻结合同模板, 定义质量标准)
   - `prompt/PHASE2_KICK_B.md` (执行指令, 直接复制 code fence 里的文本作为 prompt 发送)
3. Agent 会: 读取 Stage A 的输出 → 逐个关闭 Freeze Candidates → 冻结所有 spec → 更新 STATUS.yaml 和 EXECUTION_TRACKER.md → 输出 Builder 移交清单

### Stage B 完成后

Builder (Codex) 可以直接从 `spec/phase2/` 中读取冻结的规约开始实现. 导航入口是 `AGENTS.md` → Quick Start Task Routing → Phase 2 部分.

---

**核心要点**: 两个 stage 之间的人工审查门控是刻意设计的. Stage A 的输出质量直接决定 Stage B 能否顺利冻结. 如果 Stage A 的 Freeze Candidates 太多或方向有分歧, 先在 Stage A 会话中迭代解决, 不要急着进入 Stage B.
