//! Contract that the release workflow builds without the build standard's flags.
//!
//! A release ships, so it stays on the platform linker and the stable
//! toolchain: `cross +stable` refuses the nightly-only `-Zthreads`, and an
//! assigned `RUSTFLAGS` (even a plain `-D warnings`) displaces every
//! `rustflags` source in `.cargo/config.toml`. The workflow is read at compile
//! time, so the test needs no filesystem access.

use rstest::rstest;

const RELEASE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/.github/workflows/release.yml"
));

/// Returns the lines of the step that holds a release build command.
fn release_build_step() -> Vec<&'static str> {
    let lines: Vec<&str> = RELEASE.lines().collect();
    let Some(at) = lines
        .iter()
        .position(|line| line.contains("build --release"))
    else {
        return Vec::new();
    };
    let start = lines
        .iter()
        .take(at)
        .rposition(|line| line.trim_start().starts_with("- name:"))
        .unwrap_or(0);
    lines
        .iter()
        .skip(start)
        .take(at - start + 1)
        .copied()
        .collect()
}

/// Returns the complaint about a release build step, if any: it must assign
/// `RUSTFLAGS` once, deny warnings as whole flags (a lookalike or a comment does
/// not count), and take neither the parallel frontend nor mold.
fn release_flags_problem(step: &[&str]) -> Option<String> {
    if step.is_empty() {
        return Some("the release workflow no longer builds with `build --release`".to_owned());
    }
    let assigned: Vec<&str> = step
        .iter()
        .copied()
        .filter(|line| line.trim_start().starts_with("RUSTFLAGS:"))
        .collect();
    let [line] = assigned.as_slice() else {
        return Some(format!(
            "the release build must assign RUSTFLAGS once: {step:?}"
        ));
    };
    let words: Vec<&str> = line
        .trim_start()
        .trim_start_matches("RUSTFLAGS:")
        .split_whitespace()
        .take_while(|word| !word.starts_with('#'))
        .map(|word| word.trim_matches(['"', '\'']))
        .collect();
    let takes_a_standard_flag = words
        .iter()
        .any(|word| word.contains("-Zthreads") || word.contains("mold"));
    let denies_warnings =
        words.contains(&"-Dwarnings") || words.windows(2).any(|pair| pair == ["-D", "warnings"]);
    if takes_a_standard_flag {
        Some(format!("the release takes a standard flag: {line}"))
    } else if !denies_warnings {
        Some(format!("the release no longer denies warnings: {line}"))
    } else {
        None
    }
}

/// Scenario: the release workflow's build step.
///
/// Invariant: it assigns `RUSTFLAGS` itself, the value denies warnings, and it
/// names neither the parallel frontend nor mold.
#[test]
fn the_release_build_assigns_rustflags_without_the_standard_flags() {
    let step = release_build_step();
    assert_eq!(release_flags_problem(&step), None, "step: {step:?}");
}

/// Scenario: release build steps that keep or lose the policy.
///
/// Invariant: only a step that assigns `RUSTFLAGS` once, denies warnings and
/// takes no standard flag passes, so a rule that never fires cannot hide behind
/// this repository's own compliant workflow.
#[rstest]
#[case::denying_warnings(&["- name: Build", "  env:", "    RUSTFLAGS: -D warnings"], false)]
#[case::denying_warnings_joined(&["- name: Build", "  env:", "    RUSTFLAGS: -Dwarnings"], false)]
#[case::a_comment_after_the_policy(&["- name: Build", "    RUSTFLAGS: -D warnings # not mold"], false)]
#[case::no_assignment(&["- name: Build", "  run: cross +stable build --release"], true)]
#[case::two_assignments(&["    RUSTFLAGS: -D warnings", "    RUSTFLAGS: -D warnings"], true)]
#[case::takes_the_frontend_flag(&["    RUSTFLAGS: -D warnings -Zthreads=8"], true)]
#[case::takes_mold(&["    RUSTFLAGS: -D warnings -Clink-arg=-fuse-ld=mold"], true)]
#[case::denies_nothing(&["    RUSTFLAGS: -A warnings"], true)]
#[case::an_empty_policy(&["    RUSTFLAGS: \"\""], true)]
#[case::a_lookalike_policy(&["    RUSTFLAGS: -D warnings-extra"], true)]
#[case::the_policy_only_in_a_comment(&["    RUSTFLAGS: # -D warnings"], true)]
#[case::no_step_at_all(&[], true)]
fn a_release_step_must_keep_the_warning_policy_and_the_platform_flags(
    #[case] step: &[&str],
    #[case] refused: bool,
) {
    assert_eq!(
        release_flags_problem(step).is_some(),
        refused,
        "step: {step:?}"
    );
}
