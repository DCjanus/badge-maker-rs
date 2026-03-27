//! 对 `anafanafo` 的 Rust 侧复刻入口。
//!
//! 参考来源：
//! - `metabolize/anafanafo`
//! - `metabolize/anafanafo/packages/anafanafo`
//! - `metabolize/anafanafo/packages/char-width-table-consumer`
//!
//! 当前目标是先复刻上游对外行为与语义边界，而不是扩展成通用文本测量库。

use core::fmt;
use std::sync::OnceLock;

/// 复刻上游 `anafanafo` 当前支持的有限字体集合。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Font {
    Verdana10,
    Verdana10Bold,
    Verdana11,
    Helvetica11Bold,
}

impl Font {
    /// 返回上游 Node.js 侧使用的字体描述字符串。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Verdana10 => "10px Verdana",
            Self::Verdana10Bold => "bold 10px Verdana",
            Self::Verdana11 => "11px Verdana",
            Self::Helvetica11Bold => "bold 11px Helvetica",
        }
    }
}

/// 文本测量的输入选项。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeasureOptions {
    pub font: Font,
    pub guess: bool,
}

impl MeasureOptions {
    pub const fn new(font: Font) -> Self {
        Self { font, guess: true }
    }

    pub const fn with_guess(mut self, guess: bool) -> Self {
        self.guess = guess;
        self
    }
}

impl Default for MeasureOptions {
    fn default() -> Self {
        Self::new(Font::Verdana11)
    }
}

/// `anafanafo` 兼容层返回的错误。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    MissingWidth { char_code: u32 },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingWidth { char_code } => {
                write!(f, "No width available for character code {char_code}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// 压缩字符宽度表的消费接口。
#[derive(Clone, Debug)]
pub struct CharWidthTableConsumer {
    data: Box<[WidthTableRange]>,
    em_width: f32,
}

impl CharWidthTableConsumer {
    pub fn create(data: Vec<WidthTableRange>) -> Self {
        debug_assert!(is_valid_width_table(&data));

        let data = data.into_boxed_slice();
        let em_width = data
            .iter()
            .find(|range| range.contains('m' as u32))
            .map(|range| range.width)
            .expect("anafanafo width table must contain `m`");

        Self { data, em_width }
    }

    pub fn width_of(&self, text: &str, options: WidthOfOptions) -> Result<f32, Error> {
        text.chars().try_fold(0.0, |accum_width, ch| {
            let char_code = ch as u32;
            match self.width_of_char_code(char_code) {
                Some(char_width) => Ok(accum_width + char_width),
                None if options.guess => Ok(accum_width + self.em_width),
                None => Err(Error::MissingWidth { char_code }),
            }
        })
    }

    pub fn width_of_char_code(&self, char_code: u32) -> Option<f32> {
        if is_control_char(char_code) {
            return Some(0.0);
        }

        let index = self.data.partition_point(|range| range.lower <= char_code);
        if index == 0 {
            return None;
        }

        let candidate = &self.data[index - 1];
        candidate.contains(char_code).then_some(candidate.width)
    }
}

/// `width_of()` 的可选参数。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WidthOfOptions {
    pub guess: bool,
}

impl WidthOfOptions {
    pub const fn new() -> Self {
        Self { guess: true }
    }

    pub const fn with_guess(mut self, guess: bool) -> Self {
        self.guess = guess;
        self
    }
}

impl Default for WidthOfOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// 压缩宽度表中的单个区间。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WidthTableRange {
    pub lower: u32,
    pub upper: u32,
    pub width: f32,
}

impl WidthTableRange {
    pub const fn new(lower: u32, upper: u32, width: f32) -> Self {
        Self {
            lower,
            upper,
            width,
        }
    }

    const fn contains(self, char_code: u32) -> bool {
        self.lower <= char_code && char_code <= self.upper
    }
}

/// 测量文本宽度。
///
/// 这是对上游 `anafanafo(text, { font, guess })` 的 Rust 风格包装。
pub fn measure(text: &str, options: MeasureOptions) -> Result<f32, Error> {
    builtin_consumer(options.font).width_of(
        text,
        WidthOfOptions {
            guess: options.guess,
        },
    )
}

fn is_control_char(char_code: u32) -> bool {
    char_code <= 31 || char_code == 127
}

fn is_valid_width_table(data: &[WidthTableRange]) -> bool {
    data.iter().enumerate().all(|(index, current)| {
        current.lower <= current.upper
            && if index == 0 {
                true
            } else {
                let previous = data[index - 1];
                previous.upper < current.lower
            }
    })
}

fn builtin_consumer(font: Font) -> &'static CharWidthTableConsumer {
    match font {
        Font::Verdana10 => consumer_from_json(
            &VERDANA_10_CONSUMER,
            include_str!("../data/anafanafo/verdana-10px-normal.json"),
        ),
        Font::Verdana10Bold => consumer_from_json(
            &VERDANA_10_BOLD_CONSUMER,
            include_str!("../data/anafanafo/verdana-10px-bold.json"),
        ),
        Font::Verdana11 => consumer_from_json(
            &VERDANA_11_CONSUMER,
            include_str!("../data/anafanafo/verdana-11px-normal.json"),
        ),
        Font::Helvetica11Bold => consumer_from_json(
            &HELVETICA_11_BOLD_CONSUMER,
            include_str!("../data/anafanafo/helvetica-11px-bold.json"),
        ),
    }
}

fn consumer_from_json(
    slot: &'static OnceLock<CharWidthTableConsumer>,
    raw_json: &'static str,
) -> &'static CharWidthTableConsumer {
    slot.get_or_init(|| CharWidthTableConsumer::create(parse_width_table(raw_json)))
}

fn parse_width_table(raw_json: &str) -> Vec<WidthTableRange> {
    let rows: Vec<(u32, u32, f32)> =
        serde_json::from_str(raw_json).expect("embedded anafanafo width table must be valid JSON");

    rows.into_iter()
        .map(|(lower, upper, width)| WidthTableRange::new(lower, upper, width))
        .collect()
}

static VERDANA_10_CONSUMER: OnceLock<CharWidthTableConsumer> = OnceLock::new();
static VERDANA_10_BOLD_CONSUMER: OnceLock<CharWidthTableConsumer> = OnceLock::new();
static VERDANA_11_CONSUMER: OnceLock<CharWidthTableConsumer> = OnceLock::new();
static HELVETICA_11_BOLD_CONSUMER: OnceLock<CharWidthTableConsumer> = OnceLock::new();
