#[path = "options.rs"]
mod options;
#[path = "runner_core.rs"]
mod runner_core;

pub use options::to_badge_options;
pub use runner_core::ReferenceCase;

use runner_core::{ReferenceResult, run_reference_batch};

pub fn reference_svg_for_case(case: &ReferenceCase) -> String {
    let reference_results = run_reference_batch(std::slice::from_ref(case));
    match &reference_results[0] {
        ReferenceResult {
            id,
            ok: true,
            output: Some(output),
            error: None,
        } => {
            assert_eq!(id, &case.id);
            output.clone()
        }
        other => panic!(
            "unexpected reference result for case `{}`: {other:?}",
            case.id
        ),
    }
}
