use std::fs;

use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};

#[path = "options.rs"]
mod options;
#[path = "runner_core.rs"]
mod runner_core;

pub use options::to_badge_options;
pub use runner_core::{ReferenceCase, ReferenceResult, run_reference_batch};

use runner_core::repo_root;

pub fn load_cases() -> Vec<ReferenceCase> {
    let text = fs::read_to_string(repo_root().join("tests/data/badge_maker_cases.json"))
        .expect("failed to read badge-maker cases");
    serde_json::from_str(&text).expect("failed to parse badge-maker cases")
}

pub fn render_svg_to_rgba(svg: &str) -> Vec<u8> {
    let tree = Tree::from_str(svg, &Options::default()).expect("failed to parse SVG with usvg");
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).expect("failed to create pixmap");
    resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
    pixmap.data().to_vec()
}
