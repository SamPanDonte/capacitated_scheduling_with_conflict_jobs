#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    clippy::as_conversions,
    clippy::expect_used,
    clippy::redundant_type_annotations,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unwrap_used
)]

pub mod algo;
pub mod core;
pub mod data;

/// Runs the given scheduler on the instance read from stdin and writes the schedule to stdout.
/// Also writes the score to stdout.
/// Returns an error if the instance could not be read or the schedule could not be written.
///
/// # Errors
/// - If the instance could not be read from stdin.
/// - If the schedule could not be written to stdout.
///
/// # Panics
///  - If the schedule is invalid in debug mode.
pub fn run_from_stdin<T: core::Scheduler>(scheduler: T) -> anyhow::Result<()> {
    let instance: core::Instance = data::from_stdin()?;
    let schedule = scheduler.schedule(&instance);

    debug_assert!(schedule.verify(), "Schedule is invalid: {schedule:?}");

    data::to_stdout(&schedule)?;
    println!("{}", schedule.calculate_score());

    Ok(())
}
