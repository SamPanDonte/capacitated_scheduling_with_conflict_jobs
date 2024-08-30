use capacitated_scheduling_with_conflicts::core::Scheduler;
use capacitated_scheduling_with_conflicts::data::run;
use clap::Parser;

#[derive(Debug, Parser)]
struct Config {
    /// The input directory
    input_dir: String,
    /// Exclude scheduling algorithms
    exclude: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    for mut scheduler in schedulers(&config.exclude) {
        let report = run(&config.input_dir, false, scheduler.as_mut())?;
        print!("{report}");
    }
    Ok(())
}

fn schedulers(exclude: &[String]) -> impl Iterator<Item = Box<dyn Scheduler>> + '_ {
    capacitated_scheduling_with_conflicts::algo::SCHEDULERS
        .iter()
        .map(|init| init())
        .filter(|scheduler| !exclude.iter().any(|name| name == scheduler.name()))
}
