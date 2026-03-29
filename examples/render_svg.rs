use std::io::{self, Write};

use badge_maker_rs::{BadgeOptions, NamedColor, Style, make_badge};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg = make_badge(
        &BadgeOptions::new("passing")
            .label("example")
            .color(NamedColor::Brightgreen)
            .style(Style::Flat)
            .build(),
    )?;

    io::stdout().write_all(svg.as_bytes())?;
    Ok(())
}
