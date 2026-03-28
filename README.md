# badge-maker-rs

`badge-maker-rs` 是一个实验性 Rust 项目，目标是尽可能复刻 [Shields.io `badge-maker`](https://github.com/badges/shields/tree/master/badge-maker) 的核心行为，并最终作为一个可独立发布的 Rust crate 使用。

当前已经不只是“底层验证”阶段：仓库内已有一个可运行的 Rust 渲染实现，覆盖上游 `badge-maker` 的 5 种官方样式，并通过对照测试持续验证 SVG 输出和栅格化结果。

创建这个仓库的直接动机，是为了解决 [deps.rs issue #245](https://github.com/deps-rs/deps.rs/issues/245) 中关于 badge 样式兼容性的需求。更具体地说，我们希望让 deps.rs 能够生成尽可能兼容 Shields.io 风格与行为的 badge，而不是继续维护一套独立且差异逐渐扩大的实现。

选择复刻 Shields.io `badge-maker` 的原因也很直接：在今天的开源生态里，Shields.io 已经几乎是 badge 的事实标准。遵循它的风格和行为，有几个很实际的好处：

- 可以降低 deps.rs 使用者的心智负担，因为大多数用户已经熟悉 Shields 风格的 badge
- 可以让 deps.rs badge 更自然地和其他服务生成的 badge 保持一致
- 可以避免在同一个 README 中出现过多彼此不协调的 badge 视觉风格

## 当前阶段目标

- 继续扩展并稳固 `anafanafo` 等价模块的行为覆盖
- 继续吸收上游 `badge-maker` 测试素材，扩大高价值边界 case 集
- 维持 SVG 字符串、栅格化结果、文档预览三条校验链的一致性
- 在不牺牲架构清晰度的前提下，逐步逼近上游视觉与行为结果

## 项目状态

项目仍处于实验阶段，但核心渲染链路已经成形。

- README 暂时以中文维护
- 目标优先级为“视觉一致”，不是一开始就追求字节一致
- `anafanafo` 的第一阶段复刻已经基本落地，目前以内置宽度表和 Rust 实现提供等价模块；宽度 JSON 作为源数据保留，并在构建期生成 Rust 静态表，避免运行时 JSON 解析
- 已建立基于 Bun 运行上游 npm 包的对照测试体系，用于持续验证行为兼容性，并尽量避免在仓库内保留常驻 `node_modules`
- `badge-maker` 当前已覆盖 5 种官方样式的核心 SVG 输出路径，并通过 SVG 逐字节对照与栅格化像素对照持续回归
- 当前公开接口已经收敛为 Rust 风格的 `BadgeOptions` + `make_badge`，不再追求上游 JavaScript 校验层、JSON 输出路径等非目标接口的一致性
- rustdoc 中的 style 预览 SVG 已独立落库，并由自动化测试校验与当前实现逐字节一致
- 后续是否发布正式 crate，将根据实验结果决定

## 测试体系

当前全量测试默认通过 `just test` 运行，并尽量以集成测试和上游对照为主，而不是大量锁定内部细节的单元测试。

当前主要有三层校验：

- 基于 Bun 调用上游 `badge-maker`，逐字节对照 SVG 输出
- 使用 `resvg` 栅格化上游 SVG 和 Rust SVG，逐像素对照渲染结果
- 校验文档中引用的 style preview SVG 与当前 Rust 实现重新生成的结果逐字节一致

这几层组合的目的，是尽量用高覆盖率集成测试确保我们关心的公开行为与视觉结果稳定贴近上游。

对于 `anafanafo` 这类稳定数据驱动模块，当前也尽量避免把运行时成本留到线上路径：

- 上游宽度 JSON 继续保留在仓库中，便于审阅、更新和声明来源
- `build.rs` 会在构建期把这些 JSON 转成 Rust 静态表
- 运行时直接消费静态表，而不是再做 JSON 解析

## 当前公开 API

当前 crate 根只暴露这几个与渲染直接相关的类型和函数：

- `BadgeOptions`
- `Style`
- `Error`
- `make_badge`

最小使用方式如下：

```rust
use badge_maker_rs::{BadgeOptions, Style, make_badge};

let mut options = BadgeOptions::new("passing");
options.label = "build".to_owned();
options.color = Some("brightgreen".to_owned());
options.style = Style::Flat;

let svg = make_badge(&options)?;
assert!(svg.starts_with("<svg "));
# Ok::<(), badge_maker_rs::Error>(())
```

当前兼容边界也已经比较明确：

- 我们对齐的是 SVG 行为与视觉结果，而不是 Node.js 包的全部外部接口
- 会持续对照上游 `badge-maker` 的 SVG 输出与栅格化结果
- 不提供上游 `ValidationError`、对象字段校验包装、JSON 输出或其它 Node 特定入口
- 如 `logo_width` 这类字段，如果保留在 Rust API 中，会明确视为 Rust 侧扩展，而不是上游公开接口兼容承诺

## 参考项目

- [Shields.io](https://github.com/badges/shields)
- [badge-maker](https://github.com/badges/shields/tree/master/badge-maker)
- [anafanafo](https://github.com/metabolize/anafanafo)

## 许可证

本仓库当前使用 MIT 协议。
