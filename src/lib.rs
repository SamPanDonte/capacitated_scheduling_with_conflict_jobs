#![deny(clippy::all, clippy::cargo, clippy::expect_used, clippy::unwrap_used)]
#![deny(clippy::pedantic, clippy::nursery, unsafe_code)]
#![warn(clippy::unimplemented, clippy::redundant_type_annotations)]

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
pub fn run_from_stdin<T: core::Scheduler>(mut scheduler: T) -> anyhow::Result<()> {
    let instance: core::Instance = data::from_stdin()?;
    let schedule = scheduler.schedule(&instance);

    debug_assert!(schedule.verify(), "Schedule is invalid: {schedule:?}");

    data::to_stdout(&schedule)?;
    println!("{}", schedule.calculate_score());

    Ok(())
}

/// Macro to create a binary that reads an instance from stdin,
/// schedules it and writes the schedule to stdout.
#[macro_export]
macro_rules! binary_main {
    ($input:expr) => {
        use capacitated_scheduling_with_conflicts::{algo, run_from_stdin};

        fn main() -> anyhow::Result<()> {
            run_from_stdin($input)
        }
    };
}

#[cfg(not(target_pointer_width = "64"))]
compile_error!("Must be 64-bit system!");

/// Casts the given value to `usize`.
/// It should never fail on 64-bit systems.
///
/// # Panics
/// - If the value cannot be cast to `usize`.
#[must_use]
pub fn cast_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or_else(|_| unreachable!("Must be 64-bit system!"))
}

/// Casts the given value to `u64`.
/// It should never fail on 64-bit systems.
///
/// # Panics
/// - If the value cannot be cast to `usize`.
#[must_use]
pub fn cast_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or_else(|_| unreachable!("Must be 64-bit system!"))
}
