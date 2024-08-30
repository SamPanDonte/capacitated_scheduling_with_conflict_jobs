use capacitated_scheduling_with_conflicts::core::{Conflict, Instance, Task};
use capacitated_scheduling_with_conflicts::{cast_u64, data};
use clap::Parser;
use rand::seq::IteratorRandom;
use rand::Rng;
use std::io::Write;
use std::num::NonZero;

#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = "Generates test cases for the scheduling problem."
)]
pub struct Config {
    /// The number of processors.
    pub processors: NonZero<usize>,
    /// The number of tasks.
    pub tasks: NonZero<usize>,
    /// The maximum processing time of a task.
    pub max_time: NonZero<u64>,
    /// The deadline ratio.
    /// Deadline is computed: `max_time` * `tasks` * `deadline_ratio` / (`processors` * 2.0).
    #[clap(short, long, default_value = "1.0")]
    pub deadline_ratio: f64,
    /// Conflict ratio. 1.0 means that all tasks are in conflict with each other.
    #[clap(short, long, default_value = "0.5")]
    pub conflict_ratio: f64,
    /// Whether all tasks have the same processing time.
    #[clap(short, long, default_value = "false")]
    pub same_duration: bool,
    /// Number of test cases to generate.
    #[clap(short, long, default_value = "1")]
    pub amount: NonZero<u64>,
    /// Path to output the generated instances. If the directory does not exist, it will be created.
    #[clap(short, long, default_value = "output")]
    pub output: String,
}

fn gen(config: &Config) -> anyhow::Result<()> {
    let output = std::path::Path::new(&config.output);
    if !output.exists() {
        std::fs::create_dir_all(output)?;
    }

    for i in 0..config.amount.get() {
        let instance = Instance::new(
            config.processors.get(),
            compute_deadline(
                config.max_time.get(),
                config.tasks.get(),
                config.processors.get(),
                config.deadline_ratio,
            ),
            gen_tasks(
                config.tasks.get(),
                config.max_time.get(),
                config.same_duration,
            ),
            gen_conflicts(config.tasks.get(), config.conflict_ratio),
        );
        let filename = format!(
            "{}_0_{i}{}.in",
            config.processors,
            if config.same_duration { "_unit" } else { "" }
        );
        std::fs::File::create(output.join(filename))?
            .write_all(data::to_string(&instance)?.as_bytes())?;
    }

    Ok(())
}

fn compute_deadline(max_time: u64, tasks_number: usize, processors: usize, ratio: f64) -> u64 {
    ((max_time * cast_u64(tasks_number)) as f64 * ratio / (processors * 2) as f64).ceil() as u64
}

fn gen_tasks(tasks_number: usize, max_time: u64, unit: bool) -> Vec<Task> {
    let mut rng = rand::thread_rng();
    let mut tasks = Vec::with_capacity(tasks_number);
    for _ in 0..tasks_number {
        let time = if unit {
            max_time
        } else {
            rng.gen_range(1..=max_time)
        };
        let weight = rng.gen_range(1..=100);
        tasks.push(Task { time, weight });
    }
    tasks
}

fn gen_conflicts(tasks: usize, ratio: f64) -> Vec<Conflict> {
    let all = (tasks * (tasks - 1)) / 2;
    let required = (all as f64 / ratio).ceil() as usize;
    (0..all)
        .map(|i| Conflict::new(i / tasks, i % tasks))
        .choose_multiple(&mut rand::thread_rng(), required)
}

fn main() -> anyhow::Result<()> {
    gen(&Config::parse())
}
