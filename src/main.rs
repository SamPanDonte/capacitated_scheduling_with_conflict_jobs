use clap::{Parser, ValueEnum};
use cspcj::core::{Conflict, Instance, Scheduler, Task};
use cspcj::{algo, cast_u64, data, run_reader};
use rand::prelude::*;
use std::io::Write;
use std::num::NonZero;

#[derive(Copy, Clone, Debug)]
struct Algorithm(usize, &'static str);

impl From<Algorithm> for Box<dyn Scheduler> {
    fn from(value: Algorithm) -> Box<dyn Scheduler> {
        algo::SCHEDULERS[value.0]()
    }
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}

impl ValueEnum for Algorithm {
    fn value_variants<'a>() -> &'a [Self] {
        static ALGORITHMS: std::sync::LazyLock<Vec<Algorithm>> = std::sync::LazyLock::new(|| {
            let iter = algo::SCHEDULERS.iter().enumerate();
            iter.map(|(i, init)| Algorithm(i, init().name())).collect()
        });

        ALGORITHMS.as_slice()
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(clap::builder::PossibleValue::new(self.1))
    }
}

/// Application solving the capacitated scheduling problem.
#[derive(Debug, Parser)]
enum Application {
    /// Run one of the implemented algorithms.
    Run { algorithm: Algorithm },
    /// Run benchmarks on a set of instances.
    Bench {
        /// The input directory.
        input: String,
        /// Exclude scheduling algorithms.
        #[clap(short, long, value_delimiter = ',')]
        exclude: Vec<Algorithm>,
    },
    /// Generate test cases for the scheduling problem.
    Gen {
        /// The number of processors.
        processors: NonZero<usize>,
        /// The number of tasks.
        tasks: NonZero<usize>,
        /// The maximum processing time of a task.
        max_time: NonZero<u64>,
        /// The deadline ratio.
        /// Deadline is computed: `max_time` * `tasks` * `deadline_ratio` / (`processors` * 2.0).
        #[clap(short, long, default_value = "1.0")]
        deadline_ratio: f64,
        /// Conflict ratio. 1.0 means that all tasks are in conflict with each other.
        #[clap(short, long, default_value = "0.5")]
        conflict_ratio: f64,
        /// Whether all tasks have the same processing time.
        #[clap(short, long, default_value = "false")]
        same_duration: bool,
        /// Number of test cases to generate.
        #[clap(short, long, default_value = "1")]
        amount: NonZero<u64>,
        /// Maximum weight of a task.
        #[clap(short, long, default_value = "5")]
        max_weight: NonZero<u64>,
        /// Path to output the generated instances. If the directory does not exist, it will be created.
        #[clap(short, long, default_value = "output")]
        output: String,
    },
}

fn schedulers(exclude: &[Algorithm]) -> impl Iterator<Item = Box<dyn Scheduler>> + '_ {
    let iter = algo::SCHEDULERS.iter().map(|init| init());
    iter.filter(|scheduler| !exclude.iter().any(|name| name.1 == scheduler.name()))
}

fn compute_deadline(max_time: u64, tasks_number: usize, processors: usize, ratio: f64) -> u64 {
    ((max_time * cast_u64(tasks_number)) as f64 * ratio / (processors * 2) as f64).ceil() as u64
}

fn gen_tasks(tasks_number: usize, max_time: u64, max_weight: u64, unit: bool) -> Vec<Task> {
    let mut rng = thread_rng();
    let mut tasks = Vec::with_capacity(tasks_number);
    for _ in 0..tasks_number {
        let time = if unit {
            max_time
        } else {
            rng.gen_range(1..=max_time)
        };
        let weight = rng.gen_range(1..=max_weight);
        tasks.push(Task { time, weight });
    }
    tasks
}

fn gen_conflicts(tasks: usize, ratio: f64) -> Vec<Conflict> {
    let required = (((tasks * (tasks - 1)) / 2) as f64 * ratio).ceil() as usize;
    (0..tasks)
        .flat_map(|i| std::iter::repeat(i).zip(i + 1..tasks))
        .map(|(i, j)| Conflict::new(i, j))
        .choose_multiple(&mut thread_rng(), required)
}

fn estimate_result(instance: &Instance, unit: bool) -> anyhow::Result<u64> {
    if unit {
        algo::PolynomialTime.estimate_upper_bound(instance)
    } else {
        #[cfg(feature = "gurobi")]
        return algo::ILP2.estimate_upper_bound(instance, 60.0);
        #[cfg(not(feature = "gurobi"))]
        return Ok(instance.tasks.iter().map(|t| t.weight).sum::<u64>());
    }
}

fn main() -> anyhow::Result<()> {
    match Application::parse() {
        Application::Run { algorithm } => {
            let mut scheduler = Box::<dyn Scheduler>::from(algorithm);
            run_reader(scheduler.as_mut(), &mut std::io::stdin().lock())
        }
        Application::Bench { input, exclude } => {
            for mut scheduler in schedulers(&exclude) {
                println!("{}", data::run(&input, 0, scheduler.as_mut())?);
            }
            Ok(())
        }
        Application::Gen {
            processors,
            tasks,
            max_time,
            deadline_ratio,
            conflict_ratio,
            same_duration,
            amount,
            max_weight,
            output,
        } => {
            let processors = processors.get();
            let tasks = tasks.get();
            let max_time = max_time.get();

            let output = std::path::Path::new(&output);
            if !output.try_exists()? {
                std::fs::create_dir_all(output)?;
            }

            for i in 0..amount.get() {
                let instance = Instance::new(
                    processors,
                    compute_deadline(max_time, tasks, processors, deadline_ratio),
                    gen_tasks(tasks, max_time, max_weight.get(), same_duration),
                    gen_conflicts(tasks, conflict_ratio),
                );
                let estimate = estimate_result(&instance, same_duration)?;
                let filename = format!(
                    "{processors}_{estimate}_{i}{}.in",
                    if same_duration { "_unit" } else { "" }
                );
                std::fs::File::create(output.join(filename))?
                    .write_all(data::to_string(&instance)?.as_bytes())?;
            }
            Ok(())
        }
    }
}
