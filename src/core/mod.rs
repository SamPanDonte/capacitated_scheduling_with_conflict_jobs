mod problem;
mod solution;
mod util;

pub use problem::*;
pub use solution::*;
pub use util::*;

/// Schedules the tasks of an instance.
pub trait Scheduler {
    /// Schedules the tasks of the given instance.
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a>;

    /// Returns whether the scheduler handles non-unit tasks.
    fn non_unit(&self) -> bool {
        true
    }

    /// Returns the name of the scheduler.
    fn name(&self) -> &'static str;
}
