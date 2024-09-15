use super::matching::{gabow_algo, Graph};
use crate::cast_usize;
use crate::core::{Instance, Schedule, ScheduleInfo, Scheduler};
use anyhow::anyhow;

/// Polynomial time algorithm for the problem.
/// It is based on the maximum weighted matching in general graphs.
/// It solves the problem in `O(n^3)` time complexity using Gabow's algorithm.
/// For more than two machines, it founds an approximate solution.
///
/// # Panics
/// - If the instance tasks have different processing times.
#[derive(Clone, Debug, Default)]
pub struct PolynomialTime;

impl PolynomialTime {
    /// Estimate the upper bound of the instance.
    ///
    /// # Errors
    /// - If the instance tasks have different processing times.
    pub fn estimate_upper_bound(&mut self, instance: &Instance) -> anyhow::Result<u64> {
        if instance.tasks.is_empty() {
            return Ok(0);
        }

        let time = instance.tasks[0].time;
        if instance.tasks.iter().any(|task| task.time != time) {
            return Err(anyhow!("All tasks must have the same processing time"));
        }

        let schedule = self.schedule(instance);
        let score = schedule.calculate_score();

        Ok(score * instance.processors as u64 / 2)
    }
}

impl Scheduler for PolynomialTime {
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a> {
        polynomial_time(instance)
    }

    fn non_unit(&self) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        "PolynomialTime"
    }
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(super::SCHEDULERS)]
static INSTANCE: fn() -> Box<dyn Scheduler> = || Box::new(PolynomialTime);

fn polynomial_time(instance: &Instance) -> Schedule {
    if instance.tasks.is_empty() {
        return Schedule::new(instance);
    }

    let time = instance.tasks[0].time;

    assert!(
        !instance.tasks.iter().any(|task| task.time != time),
        "All tasks must have the same processing time"
    );

    let mut graph = Graph::default();

    for (first, task) in instance.tasks.iter().enumerate() {
        for (second, other) in instance.tasks.iter().enumerate() {
            if second > first && !instance.graph.are_conflicted(first, second) {
                graph.add_edge(first, second, task.weight + other.weight);
            }
        }
    }

    let n = instance.tasks.len();
    let d = instance.deadline / time;

    for (i, task) in instance.tasks.iter().enumerate() {
        graph.add_edge(i, n + i, task.weight);
    }

    if n > cast_usize(d) {
        for q in 0..(n - cast_usize(d)) * 2 {
            for i in 0..n * 2 {
                graph.add_edge(i, n * 2 + q, 0);
            }
        }
    }

    let Some(matching): Option<Vec<_>> = gabow_algo(&graph, true).into_iter().collect() else {
        unreachable!("Algorithm should always return a perfect matching");
    };

    let mut schedule = Schedule::new(instance);

    let mut current_time = 0;
    for (task, &paired_task) in matching[..n].iter().enumerate() {
        if task < paired_task && paired_task < 2 * n {
            schedule.schedule(task, ScheduleInfo::new(current_time, 0));
            if paired_task < n {
                schedule.schedule(paired_task, ScheduleInfo::new(current_time, 1));
            }
            current_time += time;
        }
    }

    schedule
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::Task;
    use crate::data::samples;

    #[test]
    fn test_polynomial_time() {
        assert!(samples(2, &mut PolynomialTime).is_ok());
    }

    #[test]
    #[should_panic(expected = "All tasks must have the same processing time")]
    fn test_same_time() {
        let tasks = vec![Task { weight: 1, time: 1 }, Task { weight: 1, time: 2 }];
        let _ = polynomial_time(&Instance::new_no_conflict(2, 3, tasks));
    }
}
