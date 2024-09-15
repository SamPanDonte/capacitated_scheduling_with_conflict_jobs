use crate::core::{Instance, Machine, Schedule, ScheduleInfo, Scheduler};
use rand::prelude::*;
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Performs a genetic algorithm to solve the problem.
#[derive(Clone, Debug)]
pub struct Genetic {
    generations: usize,
    rng: StdRng,
}

impl Genetic {
    /// Creates a new genetic algorithm.
    #[must_use]
    pub fn new(seed: u64, generations: usize) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        Self { generations, rng }
    }
}

impl Default for Genetic {
    fn default() -> Self {
        let generations = 800;
        let rng = StdRng::from_entropy();
        Self { generations, rng }
    }
}

impl Scheduler for Genetic {
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a> {
        if instance.tasks.is_empty() {
            return Schedule::new(instance);
        }

        if instance.tasks.len() == 1 {
            return Solution::new(vec![0], instance).to_schedule(instance);
        }

        let mut population: Vec<_> = (0..instance.tasks.len())
            .map(|_| Solution::gen(&mut self.rng, instance))
            .collect();

        population.sort_unstable();
        population.truncate(instance.tasks.len());

        for _ in 0..self.generations {
            for i in 0..instance.tasks.len() / 3 {
                if i % 3 == 0 {
                    let parents = (
                        population[..instance.tasks.len()].choose(&mut self.rng),
                        population[..instance.tasks.len()].choose(&mut self.rng),
                    );

                    if let (Some(first), Some(second)) = parents {
                        population.push(Solution::cross(first, second, instance));
                    }
                }

                if let Some(solution) = population[..instance.tasks.len()].choose(&mut self.rng) {
                    population.push(solution.mutate(&mut self.rng, instance));
                }
            }

            population.sort_unstable();
            population.truncate(instance.tasks.len());
        }

        population[0].to_schedule(instance)
    }

    fn name(&self) -> &'static str {
        "Genetic"
    }
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(super::SCHEDULERS)]
static INSTANCE: fn() -> Box<dyn Scheduler> = || Box::new(Genetic::default());

#[derive(Clone, Debug, Eq, PartialEq)]
struct Solution {
    permutation: Vec<usize>,
    score: u64,
}

impl Solution {
    fn to_schedule<'a>(&self, instance: &'a Instance) -> Schedule<'a> {
        Self::schedule(&self.permutation, instance)
    }

    fn schedule<'a>(permutation: &[usize], instance: &'a Instance) -> Schedule<'a> {
        let mut schedule = Schedule::new(instance);
        let mut machines: BTreeSet<_> = (0..instance.processors).map(Machine::new).collect();

        let d = instance.deadline;
        for &index in permutation {
            let task = instance.tasks[index];

            if machines.first().is_some_and(|m| m.free + task.time > d) {
                continue;
            }

            let Some(mut machine) = machines.pop_first() else {
                unreachable!("No machines available");
            };

            let conflicts = instance.graph.conflicts(index).iter();
            let time = conflicts
                .filter_map(|&conflict| {
                    let info = schedule.get_schedule(conflict);
                    let info = info.map(|info| info.start + instance.tasks[conflict].time);
                    info.filter(|&time| time >= machine.free)
                })
                .max()
                .or(Some(machine.free))
                .filter(|&time| time + task.time <= d);

            if let Some(time) = time {
                schedule.schedule(index, ScheduleInfo::new(time, machine.id));
                machine.free = time + task.time;
            }

            machines.insert(machine);
        }

        schedule
    }

    fn new(permutation: Vec<usize>, instance: &Instance) -> Self {
        let score = Self::schedule(&permutation, instance).calculate_score();
        Self { permutation, score }
    }

    fn gen(rng: &mut impl RngCore, instance: &Instance) -> Self {
        let mut permutation: Vec<_> = (0..instance.tasks.len()).collect();
        permutation.shuffle(rng);
        Self::new(permutation, instance)
    }

    fn cross(first: &Self, second: &Self, instance: &Instance) -> Self {
        let mut permutation = Vec::with_capacity(first.permutation.len());

        let mut missing = vec![true; first.permutation.len()];
        let mut first_iter = first.permutation.iter();
        let mut second_iter = second.permutation.iter();

        for _ in 0..(first.permutation.len() + 1) / 2 {
            for &next in first_iter.by_ref() {
                if missing[next] {
                    permutation.push(next);
                    missing[next] = false;
                    break;
                }
            }

            for &next in second_iter.by_ref() {
                if missing[next] {
                    permutation.push(next);
                    missing[next] = false;
                    break;
                }
            }
        }

        Self::new(permutation, instance)
    }

    fn mutate(&self, rng: &mut impl RngCore, instance: &Instance) -> Self {
        let mut permutation = self.permutation.clone();

        let mut indexes = permutation.choose_multiple(rng, 2).copied();
        if let (Some(first), Some(second)) = (indexes.next(), indexes.next()) {
            permutation.swap(first, second);
        }

        Self::new(permutation, instance)
    }
}

impl PartialOrd for Solution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Solution {
    fn cmp(&self, other: &Self) -> Ordering {
        let ord = self.score.cmp(&other.score).reverse();
        if ord == Ordering::Equal {
            self.permutation.cmp(&other.permutation)
        } else {
            ord
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::samples;

    #[test]
    fn test_genetic() {
        assert!(samples(0, &mut Genetic::new(10, 120)).is_ok());
    }
}
