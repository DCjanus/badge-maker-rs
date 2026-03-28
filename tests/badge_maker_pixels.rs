use badge_maker_rs::{self};
use resvg::{
    tiny_skia::{Pixmap, Transform},
    usvg::{Options, Tree},
};
mod support;

use support::{ReferenceResult, load_cases, run_reference_batch, to_badge_options};

fn render_svg_to_rgba(svg: &str) -> Vec<u8> {
    let tree = Tree::from_str(svg, &Options::default()).expect("failed to parse SVG with usvg");
    let size = tree.size().to_int_size();
    let mut pixmap = Pixmap::new(size.width(), size.height()).expect("failed to create pixmap");
    resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());
    pixmap.data().to_vec()
}

#[test]
fn badge_maker_raster_output_matches_reference_cases() {
    let cases = load_cases();
    let reference_results = run_reference_batch(&cases);
    assert_eq!(cases.len(), reference_results.len());

    for (case, reference) in cases.iter().zip(reference_results.iter()) {
        let actual_svg =
            badge_maker_rs::make_badge(&to_badge_options(case)).expect("Rust badge render failed");

        let ReferenceResult {
            id,
            ok,
            output,
            error,
        } = reference;
        assert_eq!(id, &case.id);
        assert!(
            *ok,
            "reference runner errored for `{}`: {:?}",
            case.id, error
        );
        let expected_svg = output
            .as_deref()
            .expect("reference runner returned no SVG output");

        let actual_pixels = render_svg_to_rgba(&actual_svg);
        let expected_pixels = render_svg_to_rgba(expected_svg);

        assert_eq!(
            actual_pixels, expected_pixels,
            "rasterized pixel mismatch for case `{}`",
            case.id
        );
    }
}
