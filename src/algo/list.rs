use crate::core::{weighted_task_comparator, Instance, Schedule, ScheduleBuilder, TaskWithId};

/// Simple list scheduling algorithm.
/// Returns an initial schedule, machine schedules and tardy tasks.
pub(super) fn schedule(instance: &Instance) -> ScheduleBuilder {
    let mut schedule = ScheduleBuilder::new(instance);
    let mut machines = schedule.new_machine_free_times();

    let mut tasks: Vec<TaskWithId> = instance.tasks.iter().copied().enumerate().collect();
    tasks.sort_unstable_by(weighted_task_comparator);

    for task in tasks {
        let Some(mut machine) = machines.pop_first() else {
            unreachable!("No available machines");
        };

        let time = if schedule.in_conflict(task.0, machine.free) {
            schedule.calculate_non_conflict_time(task.0, machine.free)
        } else if machine.free + task.1.time <= instance.deadline {
            Some(machine.free)
        } else {
            None
        };

        if let Some(time) = time {
            schedule.schedule(task.0, time, machine.id);
            machine.free = time + task.1.time;
        } else {
            schedule.tardy(task.0);
        }

        machines.insert(machine);
    }

    schedule
}

/// Simple list scheduling algorithm.
#[derive(Clone, Debug, Default)]
pub struct List;

impl crate::core::Scheduler for List {
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a> {
        schedule(instance).into()
    }

    fn name(&self) -> &'static str {
        "List"
    }
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(super::SCHEDULERS)]
static INSTANCE: fn() -> Box<dyn crate::core::Scheduler> = || Box::new(List);

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::samples;

    #[test]
    fn test_list() {
        assert!(samples(0, &mut List).is_ok());
    }
}
