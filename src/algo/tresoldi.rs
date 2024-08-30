use crate::core::{Instance, Schedule, ScheduleInfo, Scheduler, TaskWithId};
use crate::{cast_u64, cast_usize};
use ahash::HashMap;
use rand::prelude::{SliceRandom, StdRng};
use rand::{Rng, SeedableRng};

struct ScheduleBuilder<'a> {
    instance: &'a Instance,
    score: u64,
    tasks: Vec<TaskWithId>,
    matrix: Vec<Vec<Option<usize>>>,
    scheduled: HashMap<usize, (usize, u64)>,
}

impl<'a> ScheduleBuilder<'a> {
    fn empty(instance: &'a Instance) -> Self {
        Self {
            instance,
            score: u64::default(),
            tasks: Vec::default(),
            matrix: Vec::default(),
            scheduled: HashMap::default(),
        }
    }

    fn random(instance: &'a Instance, rng: &mut impl Rng) -> Self {
        let mut tasks: Vec<TaskWithId> = instance.tasks.clone().into_iter().enumerate().collect();
        tasks.shuffle(rng);
        Self {
            instance,
            score: 0,
            tasks,
            matrix: vec![vec![None; instance.processors]; cast_usize(instance.deadline)],
            scheduled: HashMap::default(),
        }
    }

    fn greedy_insert(&mut self) -> bool {
        let mut change = false;

        for time in 0..cast_u64(self.matrix.len()) {
            for machine in 0..self.matrix[0].len() {
                if self.matrix[cast_usize(time)][machine].is_none() {
                    for task in &self.tasks {
                        if !self.scheduled.contains_key(&task.0)
                            && self.check_time(time, machine, task)
                            && self.check_conflicts(task, time)
                        {
                            for instant in time..(time + task.1.time) {
                                self.matrix[cast_usize(instant)][machine] = Some(task.0);
                            }
                            self.score += task.1.weight;
                            self.scheduled.insert(task.0, (machine, time));
                            change = true;
                            break;
                        }
                    }
                }
            }
        }

        change
    }

    fn local_search(&mut self) -> bool {
        let mut change = false;

        for &old in &self.tasks {
            if let Some((machine, time)) = self.scheduled.get(&old.0).copied() {
                for &task in &self.tasks {
                    if self.scheduled.contains_key(&task.0) {
                        continue;
                    }

                    if (task.1.weight > old.1.weight
                        || (task.1.weight == old.1.weight && task.1.time < old.1.time))
                        && self.check_hole(&old, &task)
                        && self.check_conflicts(&task, time)
                    {
                        for instant in time..(time + old.1.time) {
                            self.matrix[cast_usize(instant)][machine] = None;
                        }

                        for instant in time..(time + task.1.time) {
                            self.matrix[cast_usize(instant)][machine] = Some(task.0);
                        }

                        self.score = self.score - old.1.weight + task.1.weight;
                        self.scheduled.remove(&old.0);
                        self.scheduled.insert(task.0, (machine, time));
                        change = true;
                        break;
                    }
                }
            }
        }

        change
    }

    fn compact(&mut self) -> bool {
        let mut change = false;

        for &task in &self.tasks {
            if let Some((machine, time)) = self.scheduled.get(&task.0).copied() {
                let mut best_machine = machine;
                let mut best_time = time;

                for machine in 0..self.matrix[0].len() {
                    let mut free = 0;

                    for time in 0..best_time + task.1.time - 1 {
                        if self.matrix[cast_usize(time)][machine].is_none() {
                            free += 1;

                            if free == task.1.time && self.check_conflicts(&task, time + 1 - free) {
                                best_time = time - free + 1;
                                best_machine = machine;
                            }
                        } else {
                            free = 0;
                        }
                    }
                }

                if best_time < time {
                    for instant in time..(time + task.1.time) {
                        self.matrix[cast_usize(instant)][machine] = None;
                    }

                    for instant in best_time..(best_time + task.1.time) {
                        self.matrix[cast_usize(instant)][best_machine] = Some(task.0);
                    }

                    self.scheduled.insert(task.0, (best_machine, best_time));
                    change = true;
                }
            }
        }

        change
    }

    fn check_time(&self, time: u64, machine: usize, task: &TaskWithId) -> bool {
        if time + task.1.time > cast_u64(self.matrix.len()) {
            return false;
        }

        for instant in time..(time + task.1.time) {
            if self.matrix[cast_usize(instant)][machine].is_some() {
                return false;
            }
        }

        true
    }

    fn check_conflicts(&self, task: &TaskWithId, time: u64) -> bool {
        for &conflict in self.instance.graph.conflicts(task.0) {
            if let Some(&(_, other_time)) = self.scheduled.get(&conflict) {
                let other = self.instance.tasks[conflict];
                if time < other_time + other.time && other_time < time + task.1.time {
                    return false;
                }
            }
        }

        true
    }

    fn check_hole(&self, task: &TaskWithId, new_task: &TaskWithId) -> bool {
        if task.1.time >= new_task.1.time {
            return true;
        }

        let Some(&(machine, time)) = self.scheduled.get(&task.0) else {
            unreachable!("Task must be scheduled");
        };

        if time + new_task.1.time > cast_u64(self.matrix.len()) {
            return false;
        }

        let start = cast_usize(time + task.1.time);
        let end = cast_usize(time + new_task.1.time);
        for instant in &self.matrix[start..end] {
            if instant[machine].is_some() {
                return false;
            }
        }

        true
    }
}

impl<'a> From<ScheduleBuilder<'a>> for Schedule<'a> {
    fn from(value: ScheduleBuilder<'a>) -> Self {
        let mut schedule = Schedule::new(value.instance);

        for (task, (machine, time)) in value.scheduled {
            schedule.schedule(task, ScheduleInfo::new(time, machine));
        }

        schedule
    }
}

/// Tresoldi's algorithm.
#[derive(Clone, Debug)]
pub struct Tresoldi {
    iterations: usize,
    rng: StdRng,
}

impl Tresoldi {
    /// Creates a new instance of `Tresoldi`.
    #[must_use]
    pub fn new(iterations: usize, seed: u64) -> Self {
        Self {
            iterations,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(super::SCHEDULERS)]
static INSTANCE: fn() -> Box<dyn Scheduler> = || Box::new(Tresoldi::default());

impl Default for Tresoldi {
    fn default() -> Self {
        Self {
            iterations: 10,
            rng: StdRng::from_rng(rand::thread_rng()).unwrap_or_else(|_| StdRng::seed_from_u64(0)),
        }
    }
}

impl Scheduler for Tresoldi {
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a> {
        let mut best_solution = ScheduleBuilder::empty(instance);

        for _ in 0..self.iterations {
            let mut solution = ScheduleBuilder::random(instance, &mut self.rng);

            loop {
                let mut change = solution.greedy_insert();
                change |= solution.local_search();
                change |= solution.compact();

                if !change {
                    break;
                }
            }

            if solution.score > best_solution.score {
                best_solution = solution;
            }
        }

        best_solution.into()
    }

    fn name(&self) -> &'static str {
        "Tresoldi"
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::samples;

    #[test]
    fn test_tresoldi() {
        assert!(samples(false, &mut Tresoldi::new(10, 0)).is_ok());
    }
}
