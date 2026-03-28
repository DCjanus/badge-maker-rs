# badge-maker-rs

`badge-maker-rs` 是一个实验性 Rust 项目，目标是尽可能兼容 [Shields.io `badge-maker`](https://github.com/badges/shields/tree/master/badge-maker) 的最终视觉输出，并最终沉淀成一个可独立使用的 Rust crate。

它当前已经具备可运行的 Rust 渲染实现，覆盖上游 `badge-maker` 的 5 种官方样式，并通过上游对照测试持续校验 SVG 输出与栅格化结果。

创建这个仓库的直接动机，是为了解决 [deps.rs issue #245](https://github.com/deps-rs/deps.rs/issues/245) 中关于 badge 样式兼容性的需求。我们希望 deps.rs 生成的 badge 在最终渲染结果上尽可能兼容 Shields.io，而不是继续维护一套逐渐分叉的自定义实现。

## 项目状态

项目仍处于实验阶段，但核心渲染链路已经成形：

- README 暂时以中文维护
- 目标优先级是最终渲染结果的像素级一致，而不是 SVG 文本的字节一致
- `anafanafo` 的第一阶段复刻已经基本落地，目前以内置宽度表和 Rust 实现提供等价模块；宽度 JSON 作为源数据保留，并在构建期生成 Rust 静态表，避免运行时 JSON 解析
- 已建立基于 Bun 运行上游 npm 包的对照测试体系，用于持续验证视觉兼容性，并尽量避免在仓库内保留常驻 `node_modules`
- `badge-maker` 当前已覆盖 5 种官方样式的核心输出路径，并通过 SVG 对照与栅格化像素对照持续回归
- 当前公开接口已经收敛为 Rust 风格的 `BadgeOptions::new(message)` + `make_badge`；不再追求上游 JavaScript 校验层、JSON 输出路径等非目标接口的一致性
- rustdoc 中的 style 预览 SVG 已独立落库，并由自动化测试校验与当前实现逐字节一致
- 后续是否发布正式 crate，将根据实验结果决定

## 测试体系

当前全量测试默认通过 `just test` 运行，并尽量以集成测试和上游对照为主，而不是大量锁定内部细节的单元测试。

当前主要有三层校验：

- 基于 Bun 调用上游 `badge-maker`，对照 SVG 输出
- 使用 `resvg` 栅格化上游 SVG 和 Rust SVG，逐像素对照渲染结果
- 校验文档中引用的 style preview SVG 与当前 Rust 实现重新生成的结果逐字节一致

这几层组合的目的，是尽量用高覆盖率集成测试确保我们关心的公开行为与最终视觉结果稳定兼容上游。其中栅格化后的像素结果是更高优先级的真相来源，SVG 文本对照主要用于快速定位偏差。

对于 `anafanafo` 这类稳定数据驱动模块，当前也尽量避免把运行时成本留到线上路径：

- 上游宽度 JSON 继续保留在仓库中，便于审阅、更新和声明来源
- `build.rs` 会在构建期把这些 JSON 转成 Rust 静态表
- 运行时直接消费静态表，而不是再做 JSON 解析

## 当前公开 API

crate 根目前只暴露与渲染直接相关的类型和函数：

- `BadgeOptions`
- `Color`
- `NamedColor`
- `Style`
- `Error`
- `make_badge`

最小使用方式如下：

```rust
use badge_maker_rs::{BadgeOptions, Color, Style, make_badge};

let svg = make_badge(
    &BadgeOptions::new("passing")
        .label("build")
        .color("brightgreen".parse()?)
        .style(Style::Flat)
        .build(),
)?;
assert!(svg.starts_with("<svg "));
# Ok::<(), Box<dyn std::error::Error>>(())
```

当前兼容边界比较明确：

- 我们对齐的是最终渲染效果，而不是 Node.js 包的全部外部接口
- 会持续对照上游 `badge-maker` 的 SVG 输出与栅格化结果，其中像素一致性优先级更高
- 不提供上游 `ValidationError`、对象字段校验包装、JSON 输出或其它 Node 特定入口
- 如 `logo_width` 这类字段，如果保留在 Rust API 中，会明确视为 Rust 侧扩展，而不是上游公开接口兼容承诺

输入语义目前约定如下：

- `label` 和 `message` 会先做首尾空白裁剪，再参与布局
- `message` 是唯一必填输入，并在 `BadgeOptions::new(message)` 时提供
- 文本内容和属性内容在输出 SVG 时会统一做 XML 转义
- 日常调用优先推荐 `"brightgreen".parse::<Color>()`
- `color` / `label_color` 未提供时沿用 Shields 默认色
- `Color::literal(...)` 在非法时不报错，而是回退到样式默认色
- `left_link` / `right_link` 直接表达链接结构：
  仅 `left_link` 时会包住整块 badge body；仅 `right_link` 时只给右半边加链接；两者同时存在时则左右各自独立链接
- `logo_data_url` 对应 `badge-maker` 原始字段名 `logoBase64`
- `logo_width` 对应 `badge-maker` 原始字段名 `logoWidth`
- `id_suffix` 对应 `badge-maker` 原始字段名 `idSuffix`
- `id_suffix` 是当前唯一明确会返回错误的公开输入约束

## Breaking Changes

- `BadgeOptions.links` 已移除，改为 `left_link` / `right_link`
- `BadgeOptions::set_links(...)` 已移除，改为直接设置 `left_link` / `right_link`
- `BadgeOptions.logo_base64` 已重命名为 `logo_data_url`

## 参考项目

- [Shields.io](https://github.com/badges/shields)
- [badge-maker](https://github.com/badges/shields/tree/master/badge-maker)
- [anafanafo](https://github.com/metabolize/anafanafo)

## 许可证

本仓库当前使用 MIT 协议。
