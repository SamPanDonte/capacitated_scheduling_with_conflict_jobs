use super::{Instance, Schedule, ScheduleInfo, Task};
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Task with its id.
pub type TaskWithId = (usize, Task);

/// Machine is a resource that can be used to process a task.
/// It's ordered by free time.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Machine {
    pub id: usize,
    pub free: u64,
}

impl Machine {
    /// Creates a new machine with free time 0.
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self { id, free: 0 }
    }

    const fn with_free_time(id: usize, free: u64) -> Self {
        Self { id, free }
    }
}

impl PartialOrd<Self> for Machine {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Machine {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.free.cmp(&other.free) {
            Ordering::Equal => self.id.cmp(&other.id),
            order => order,
        }
    }
}

/// Compares two tasks by their weight and processing time.
#[must_use]
pub fn weighted_task_comparator(first: &TaskWithId, second: &TaskWithId) -> Ordering {
    (first.1.time * second.1.weight).cmp(&(second.1.time * first.1.weight))
}

/// A builder for creating a schedule.
/// It's used to schedule tasks on machines with utility methods.
#[derive(Clone, Debug)]
pub struct ScheduleBuilder<'a> {
    instance: &'a Instance,
    schedule: Schedule<'a>,
    machines: Vec<Vec<usize>>,
    tardies: Vec<usize>,
}

impl<'a> ScheduleBuilder<'a> {
    /// Creates a new schedule builder.
    #[must_use]
    pub fn new(instance: &'a Instance) -> Self {
        Self {
            instance,
            schedule: Schedule::new(instance),
            machines: vec![Vec::new(); instance.processors],
            tardies: Vec::new(),
        }
    }

    /// Schedules a task on a machine at a given time.
    /// Time must be within deadline and bigger than the last task.
    pub fn schedule(&mut self, id: usize, time: u64, machine: usize) {
        self.schedule.schedule(id, ScheduleInfo::new(time, machine));
        self.machines[machine].push(id);
    }

    /// Returns the schedule for a task.
    #[must_use]
    pub fn get_schedule(&self, task: usize) -> Option<&ScheduleInfo> {
        self.schedule.get_schedule(task)
    }

    /// Marks a task as tardy.
    /// It must be called for tasks that are not scheduled and are not yet tardy.
    pub fn tardy(&mut self, task: usize) {
        self.tardies.push(task);
    }

    /// Returns the number of machines.
    #[must_use]
    pub fn machines_len(&self) -> usize {
        self.machines.len()
    }

    /// Returns the number of tasks in a machine.
    #[must_use]
    pub fn machine_tasks_len(&self, machine: usize) -> usize {
        self.machines[machine].len()
    }

    /// Returns the number of tardy tasks.
    #[must_use]
    pub fn tardy_len(&self) -> usize {
        self.tardies.len()
    }

    /// Calculates the score of the schedule.
    #[must_use]
    pub fn calculate_score(&self) -> u64 {
        self.schedule.calculate_score()
    }

    /// Creates an ordered set of machines with order of free time.
    #[must_use]
    pub fn new_machine_free_times(&self) -> BTreeSet<Machine> {
        self.machines
            .iter()
            .enumerate()
            .map(|(id, tasks)| {
                let free = tasks
                    .last()
                    .and_then(|&task| self.schedule.get_schedule(task).map(|info| (task, info)))
                    .map(|(task, info)| info.start + self.instance.tasks[task].time)
                    .unwrap_or_default();
                Machine::with_free_time(id, free)
            })
            .collect()
    }

    /// Check if the given task with the given start time is in conflict with another task.
    #[must_use]
    pub fn in_conflict(&self, task: usize, time: u64) -> bool {
        self.schedule.in_conflict(task, time)
    }

    /// Calculates first available time for a task that is not in conflict with other tasks.
    /// It returns None if there is no available time within deadline.
    #[must_use]
    pub fn calculate_non_conflict_time(&self, task: usize, minimum_time: u64) -> Option<u64> {
        let task_time = self.instance.tasks[task].time;
        let mut times: Vec<_> = self
            .instance
            .graph
            .conflicts(task)
            .iter()
            .filter_map(|&other| {
                let t = self.instance.tasks[other].time;
                self.schedule.get_schedule(other).map(|info| info.start + t)
            })
            .filter(|&time| time >= minimum_time)
            .filter(|&time| time + task_time <= self.instance.deadline)
            .filter(|&time| !self.schedule.in_conflict(task, time))
            .collect();
        times.sort_unstable();
        times.first().copied()
    }

    /// Reorganizes the schedule using the given operations.
    /// It removes the tasks that are changed and fixes the machines and tardy tasks.
    /// The op function should return a tuple with machine id, index, and tardy tasks.
    pub fn reorganize_schedule<F>(&mut self, op: F)
    where
        F: FnOnce(&mut [Vec<usize>], &mut Vec<usize>) -> (Vec<(usize, usize)>, Vec<usize>),
    {
        let (machines, tardy) = op(&mut self.machines, &mut self.tardies);

        for task in tardy {
            self.schedule.remove_schedule(task);
        }

        for (machine, index) in &machines {
            for &task in &self.machines[*machine][*index..] {
                self.schedule.remove_schedule(task);
            }
        }

        for (machine, index) in machines {
            self.fix_machine(machine, index);
        }

        self.fix_tardy();
    }

    fn fix_machine(&mut self, machine: usize, index: usize) {
        let mut free = if index == 0 {
            0
        } else {
            let task = self.machines[machine][index - 1];
            self.schedule
                .get_schedule(task)
                .map(|info| info.start + self.instance.tasks[task].time)
                .unwrap_or_default()
        };

        for &task in &self.machines[machine][index..] {
            let processing_time = self.instance.tasks[task].time;
            let time = if self.schedule.in_conflict(task, free) {
                self.calculate_non_conflict_time(task, free)
            } else if free + processing_time <= self.instance.deadline {
                Some(free)
            } else {
                None
            };

            if let Some(time) = time {
                let info = ScheduleInfo::new(time, machine);
                self.schedule.schedule(task, info);
                free = time + processing_time;
            } else {
                self.tardies.push(task);
            }
        }

        self.machines[machine].retain(|&id| self.schedule.get_schedule(id).is_some());
    }

    fn fix_tardy(&mut self) {
        self.tardies.sort_unstable_by(|&a, &b| {
            weighted_task_comparator(&(a, self.instance.tasks[a]), &(b, self.instance.tasks[b]))
        });

        let mut machines = self.new_machine_free_times();
        let mut tasks = Vec::new();

        std::mem::swap(&mut self.tardies, &mut tasks);

        for task in tasks {
            let Some(mut machine) = machines.pop_first() else {
                unreachable!("Machine number is always greater than 0")
            };

            let time = if self.in_conflict(task, machine.free) {
                self.calculate_non_conflict_time(task, machine.free)
            } else if machine.free + self.instance.tasks[task].time <= self.instance.deadline {
                Some(machine.free)
            } else {
                None
            };

            if let Some(time) = time {
                self.schedule(task, time, machine.id);
                machine.free = time + self.instance.tasks[task].time;
            } else {
                self.tardy(task);
            }

            machines.insert(machine);
        }
    }
}

impl<'a> From<ScheduleBuilder<'a>> for Schedule<'a> {
    fn from(builder: ScheduleBuilder<'a>) -> Self {
        builder.schedule
    }
}
