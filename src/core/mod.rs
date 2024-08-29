mod problem;
mod solution;
mod util;

pub use problem::*;
pub use solution::*;
pub use util::*;

/// Schedules the tasks of an instance.
pub trait Scheduler: Clone {
    /// Schedules the tasks of the given instance.
    fn schedule(self, instance: &Instance) -> Schedule;
}

impl<T: Fn(&Instance) -> Schedule> Scheduler for &T {
    fn schedule(self, instance: &Instance) -> Schedule {
        self(instance)
    }
}
