# badge-maker-rs

`badge-maker-rs` 是一个实验性 Rust 项目，目标是尽可能复刻 [Shields.io `badge-maker`](https://github.com/badges/shields/tree/master/badge-maker) 的核心行为，并最终作为一个可独立发布的 Rust crate 使用。

当前阶段的重点不是完整实现所有 badge 风格，而是先把底层能力做扎实，优先验证文字宽度计算与布局相关的兼容性基础。

创建这个仓库的直接动机，是为了解决 [deps.rs issue #245](https://github.com/deps-rs/deps.rs/issues/245) 中关于 badge 样式兼容性的需求。更具体地说，我们希望让 deps.rs 能够生成尽可能兼容 Shields.io 风格与行为的 badge，而不是继续维护一套独立且差异逐渐扩大的实现。

选择复刻 Shields.io `badge-maker` 的原因也很直接：在今天的开源生态里，Shields.io 已经几乎是 badge 的事实标准。遵循它的风格和行为，有几个很实际的好处：

- 可以降低 deps.rs 使用者的心智负担，因为大多数用户已经熟悉 Shields 风格的 badge
- 可以让 deps.rs badge 更自然地和其他服务生成的 badge 保持一致
- 可以避免在同一个 README 中出现过多彼此不协调的 badge 视觉风格

## 当前阶段目标

- 继续扩展并稳固 `anafanafo` 等价模块的行为覆盖
- 建立可重复、可长期维护的对照测试方式
- 明确与官方 `badge-maker` 的兼容边界

## 项目状态

项目处于早期实验阶段。

- README 暂时以中文维护
- 目标优先级为“视觉一致”，不是一开始就追求字节一致
- `anafanafo` 的第一阶段复刻已经基本落地，目前以内置宽度表和 Rust 实现提供等价模块
- 已建立基于 Bun 运行上游 npm 包的对照测试体系，用于持续验证行为兼容性
- 后续是否发布正式 crate，将根据实验结果决定

## 参考项目

- [Shields.io](https://github.com/badges/shields)
- [badge-maker](https://github.com/badges/shields/tree/master/badge-maker)
- [anafanafo](https://github.com/metabolize/anafanafo)

## 许可证

本仓库当前使用 MIT 协议。
