use std::{fs, path::PathBuf};

use badge_maker_rs::{BadgeOptions, NamedColor, Style, make_badge};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let svg = make_badge(
        &BadgeOptions::new("passing")
            .label("example")
            .color(NamedColor::Brightgreen)
            .style(Style::Flat)
            .build(),
    )?;

    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("example-output.svg");
    fs::write(&output_path, svg)?;

    println!("Wrote {}", output_path.display());
    Ok(())
}
