use crate::core::{Instance, Schedule, ScheduleBuilder, Scheduler};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

type Neighborhood<'a, 'b> = dyn Iterator<Item = ScheduleBuilder<'a>> + 'b;

/// Neighborhood that swaps two tasks on the same machine.
pub struct SwapSingleMachine<'a, 'b> {
    schedule: &'b ScheduleBuilder<'a>,
    machine: usize,
    i: usize,
    j: usize,
}

/// Creates a new instance of `SwapSingleMachine` neighborhood.
fn swap_single_machine<'a, 'b>(schedule: &'b ScheduleBuilder<'a>) -> Box<Neighborhood<'a, 'b>> {
    Box::new(SwapSingleMachine {
        schedule,
        machine: 0,
        i: 0,
        j: 1,
    })
}

impl<'a, 'b> Iterator for SwapSingleMachine<'a, 'b> {
    type Item = ScheduleBuilder<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.machine < self.schedule.machines_len() {
            while self.i + 1 < self.schedule.machine_tasks_len(self.machine) {
                if self.j < self.schedule.machine_tasks_len(self.machine) {
                    let mut builder = self.schedule.clone();

                    builder.reorganize_schedule(|machines, _| {
                        machines[self.machine].swap(self.i, self.j);
                        (vec![(self.machine, self.i)], vec![])
                    });

                    self.j += 1;

                    return Some(builder);
                }
                self.i += 1;
            }
            self.machine += 1;
        }
        None
    }
}

/// Neighborhood that moves task on the same machine.
struct MoveSingleMachine<'a, 'b> {
    schedule: &'b ScheduleBuilder<'a>,
    machine: usize,
    i: usize,
    j: usize,
}

/// Creates a new instance of `MoveSingleMachine` neighborhood.
fn move_single_machine<'a, 'b>(schedule: &'b ScheduleBuilder<'a>) -> Box<Neighborhood<'a, 'b>> {
    Box::new(MoveSingleMachine {
        schedule,
        machine: 0,
        i: 0,
        j: 1,
    })
}

impl<'a, 'b> Iterator for MoveSingleMachine<'a, 'b> {
    type Item = ScheduleBuilder<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.machine < self.schedule.machines_len() {
            while self.i + 1 < self.schedule.machine_tasks_len(self.machine) {
                if self.j < self.schedule.machine_tasks_len(self.machine) {
                    let mut builder = self.schedule.clone();

                    builder.reorganize_schedule(|machines, _| {
                        let task = machines[self.machine].remove(self.i);
                        machines[self.machine].insert(self.j, task);
                        (vec![(self.machine, self.i.min(self.j))], vec![])
                    });

                    self.j += 1;

                    return Some(builder);
                }
                self.i += 1;
            }
            self.machine += 1;
        }
        None
    }
}

/// Neighborhood that swaps tasks on different machines.
struct SwapTwoMachines<'a, 'b> {
    schedule: &'b ScheduleBuilder<'a>,
    first: usize,
    second: usize,
    i: usize,
    j: usize,
}

/// Creates a new instance of `SwapTwoMachines` neighborhood.
fn swap_two_machines<'a, 'b>(schedule: &'b ScheduleBuilder<'a>) -> Box<Neighborhood<'a, 'b>> {
    Box::new(SwapTwoMachines {
        schedule,
        first: 0,
        second: 1,
        i: 0,
        j: 0,
    })
}

impl<'a, 'b> Iterator for SwapTwoMachines<'a, 'b> {
    type Item = ScheduleBuilder<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.first + 1 < self.schedule.machines_len() {
            while self.second < self.schedule.machines_len() {
                while self.i < self.schedule.machine_tasks_len(self.first) {
                    if self.j < self.schedule.machine_tasks_len(self.second) {
                        let mut builder = self.schedule.clone();

                        builder.reorganize_schedule(|machines, _| {
                            let value = machines[self.first][self.i];
                            machines[self.first][self.i] = machines[self.second][self.j];
                            machines[self.second][self.j] = value;

                            (vec![(self.first, self.i), (self.second, self.j)], vec![])
                        });

                        self.j += 1;

                        return Some(builder);
                    }
                    self.i += 1;
                }
                self.second += 1;
            }
            self.first += 1;
        }
        None
    }
}

/// Neighborhood that moves task on different machine.
struct MoveTwoMachines<'a, 'b> {
    schedule: &'b ScheduleBuilder<'a>,
    first: usize,
    second: usize,
    i: usize,
    j: usize,
}

/// Creates a new instance of `MoveTwoMachines` neighborhood.
fn move_two_machines<'a, 'b>(schedule: &'b ScheduleBuilder<'a>) -> Box<Neighborhood<'a, 'b>> {
    Box::new(MoveTwoMachines {
        schedule,
        first: 0,
        second: 1,
        i: 0,
        j: 0,
    })
}

impl<'a, 'b> Iterator for MoveTwoMachines<'a, 'b> {
    type Item = ScheduleBuilder<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.first + 1 < self.schedule.machines_len() {
            while self.second < self.schedule.machines_len() {
                while self.i < self.schedule.machine_tasks_len(self.first) {
                    if self.j <= self.schedule.machine_tasks_len(self.second) {
                        let mut builder = self.schedule.clone();

                        builder.reorganize_schedule(|machines, _| {
                            let value = machines[self.first].remove(self.i);
                            machines[self.second].insert(self.j, value);

                            (vec![(self.first, self.i), (self.second, self.j)], vec![])
                        });

                        self.j += 1;

                        return Some(builder);
                    }
                    self.i += 1;
                }
                self.second += 1;
            }
            self.first += 1;
        }
        None
    }
}

/// Neighborhood that replaces task with a tardy task.
struct ReplaceWithTardy<'a, 'b> {
    schedule: &'b ScheduleBuilder<'a>,
    machine: usize,
    i: usize,
    j: usize,
}

/// Creates a new instance of `ReplaceWithTardy` neighborhood.
fn replace_with_tardy<'a, 'b>(schedule: &'b ScheduleBuilder<'a>) -> Box<Neighborhood<'a, 'b>> {
    Box::new(ReplaceWithTardy {
        schedule,
        machine: 0,
        i: 0,
        j: 0,
    })
}

impl<'a, 'b> Iterator for ReplaceWithTardy<'a, 'b> {
    type Item = ScheduleBuilder<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.machine < self.schedule.machines_len() {
            while self.i < self.schedule.machine_tasks_len(self.machine) {
                if self.j < self.schedule.tardy_len() {
                    let mut builder = self.schedule.clone();

                    builder.reorganize_schedule(|machines, tardy_tasks| {
                        std::mem::swap(
                            &mut machines[self.machine][self.i],
                            &mut tardy_tasks[self.j],
                        );

                        (vec![(self.machine, self.i)], vec![tardy_tasks[self.j]])
                    });

                    self.j += 1;

                    return Some(builder);
                }
                self.i += 1;
            }
            self.machine += 1;
        }
        None
    }
}

/// Neighborhood that adds a tardy task.
struct AddTardy<'a, 'b> {
    schedule: &'b ScheduleBuilder<'a>,
    machine: usize,
    i: usize,
    j: usize,
}

/// Creates a new instance of `AddTardy` neighborhood.
fn add_tardy<'a, 'b>(schedule: &'b ScheduleBuilder<'a>) -> Box<Neighborhood<'a, 'b>> {
    Box::new(AddTardy {
        schedule,
        machine: 0,
        i: 0,
        j: 0,
    })
}

impl<'a, 'b> Iterator for AddTardy<'a, 'b> {
    type Item = ScheduleBuilder<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.machine < self.schedule.machines_len() {
            while self.i <= self.schedule.machine_tasks_len(self.machine) {
                if self.j < self.schedule.tardy_len() {
                    let mut builder = self.schedule.clone();

                    builder.reorganize_schedule(|machines, tardy_tasks| {
                        machines[self.machine].insert(self.i, tardy_tasks[self.j]);
                        tardy_tasks.remove(self.j);

                        (vec![(self.machine, self.i)], vec![])
                    });

                    self.j += 1;

                    return Some(builder);
                }
                self.i += 1;
            }
            self.machine += 1;
        }
        None
    }
}

fn neighborhood_search(mut schedule: ScheduleBuilder) -> ScheduleBuilder {
    let factories = [
        swap_single_machine,
        move_single_machine,
        swap_two_machines,
        move_two_machines,
        replace_with_tardy,
        add_tardy,
    ];

    let mut k = 0;

    while k < factories.len() {
        let mut best_score = schedule.calculate_score();
        let mut best_schedule = None;

        for schedule in factories[k](&schedule) {
            let score = schedule.calculate_score();
            if score > best_score {
                best_score = score;
                best_schedule = Some(schedule);
            }
        }

        if let Some(best_schedule) = best_schedule {
            schedule = best_schedule;
            k = 0;
        } else {
            k += 1;
        }
    }

    schedule
}

/// Performs the Variable Neighborhood Search algorithm.
/// It is done inside iterations of the Local Search algorithm.
#[derive(Clone, Debug)]
pub struct VariableNeighborhoodSearch {
    iterations: usize,
    rng: StdRng,
}

impl VariableNeighborhoodSearch {
    /// Creates a new instance of `VariableNeighborhoodSearch`.
    #[must_use]
    pub fn new(iterations: usize, seed: u64) -> Self {
        Self {
            iterations,
            rng: StdRng::seed_from_u64(seed),
        }
    }
}

impl Default for VariableNeighborhoodSearch {
    fn default() -> Self {
        Self {
            iterations: 10,
            rng: StdRng::from_rng(rand::thread_rng()).unwrap_or_else(|_| StdRng::seed_from_u64(0)),
        }
    }
}

impl Scheduler for VariableNeighborhoodSearch {
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a> {
        if instance.tasks.is_empty() {
            return Schedule::new(instance);
        }

        let mut schedule = neighborhood_search(super::list::schedule(instance));
        let mut best_score = schedule.calculate_score();

        for _ in 0..self.iterations {
            let mut new_schedule = schedule.clone();

            for _ in 0..(instance.tasks.len() / 20).max(1) {
                let task = self.rng.gen_range(0..instance.tasks.len());
                let task_machine = new_schedule.get_schedule(task).map(|info| info.processor);

                new_schedule.reorganize_schedule(|machines, tardy_tasks| {
                    let mut machine_fixings = Vec::with_capacity(2);

                    match task_machine {
                        Some(machine) => {
                            if let Some(pos) = machines[machine].iter().position(|&id| id == task) {
                                machine_fixings.push((machine, pos));
                            }
                            machines[machine].retain(|&id| id != task);
                        }
                        None => tardy_tasks.retain(|&id| id != task),
                    }

                    let new_machine = self.rng.gen_range(0..instance.processors);
                    let new_position = self.rng.gen_range(0..=machines[new_machine].len());
                    machines[new_machine].insert(new_position, task);

                    match task_machine.filter(|&machine| machine == new_machine) {
                        Some(_) => machine_fixings[0].1 = new_position.min(machine_fixings[0].1),
                        None => machine_fixings.push((new_machine, new_position)),
                    }

                    (machine_fixings, vec![])
                });
            }

            let new_schedule = neighborhood_search(new_schedule);
            let new_score = new_schedule.calculate_score();

            if new_score > best_score {
                best_score = new_score;
                schedule = new_schedule;
            }
        }

        schedule.into()
    }

    fn name(&self) -> &'static str {
        "VNS"
    }
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(super::SCHEDULERS)]
static INSTANCE: fn() -> Box<dyn Scheduler> = || Box::new(VariableNeighborhoodSearch::default());

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::samples;

    #[test]
    fn test_vns() {
        let mut vns = VariableNeighborhoodSearch::new(10, 0);
        assert!(samples(0, &mut vns).is_ok());
    }
}
