use crate::core::Scheduler;
use crate::data::deserialize;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::BufReader;

/// Report of running a directory of samples.
#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    scheduler: String,
    entries: Vec<ReportEntry>,
}

impl Report {
    /// Create a new report.
    fn new(scheduler: String) -> Self {
        let entries = Vec::new();
        Self { scheduler, entries }
    }

    /// Get the scheduler name.
    #[must_use]
    pub fn scheduler_name(&self) -> &str {
        &self.scheduler
    }

    /// Get the entries.
    #[must_use]
    pub fn entries(&self) -> &[ReportEntry] {
        &self.entries
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Scheduler: {}", self.scheduler)?;
        for entry in &self.entries {
            writeln!(f, "{entry}")?;
        }
        writeln!(f, "-------------------")
    }
}

/// Report of running a single sample.
#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize)]
pub struct ReportEntry {
    pub name: String,
    pub score: u64,
    pub time: f64,
}

impl Display for ReportEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {} in {:.2} sec", self.name, self.score, self.time)
    }
}

/// Run all samples in the `samples` directory.
/// Print the report to stdout.
///
/// # Arguments
/// - `valid` is true, check if the score is correct.
/// - `solver` is the scheduler to run.
///
/// # Errors
/// - If a file cannot be read.
/// - If no samples are found.
///
/// # Panics
/// - If the schedule is invalid.
/// - If the score is incorrect and `score` is true.
pub fn samples(valid: bool, solver: &mut dyn Scheduler) -> anyhow::Result<()> {
    run("samples", valid, solver).and_then(|report| {
        if report.entries.is_empty() {
            Err(anyhow!("No samples found"))
        } else {
            println!("{report}");
            Ok(())
        }
    })
}

/// Run all samples in the `dir` directory.
///
/// # Arguments
/// - `score` is true, check if the score is correct.
/// - `solver` is the scheduler to run.
///
/// # Errors
/// - If a file cannot be read.
///
/// # Panics
/// - If the schedule is invalid.
/// - If the score is incorrect.
pub fn run(dir: &str, valid: bool, solver: &mut dyn Scheduler) -> anyhow::Result<Report> {
    let mut report = Report::new(solver.name().into());

    for file in std::fs::read_dir(dir)? {
        let file = file?;
        let (name, machines, result, is_unit) = parse_filename(&file.file_name())?;

        if (!solver.non_unit() || is_unit) && machines <= solver.maximum_machine() {
            let instance = deserialize(&mut BufReader::new(File::open(file.path())?))?;

            let time = std::time::Instant::now();
            let schedule = solver.schedule(&instance);
            let time = time.elapsed().as_secs_f64();

            assert!(schedule.verify(), "Invalid schedule created");

            let score = schedule.calculate_score();
            if valid {
                assert_eq!(score, result, "Invalid score {name}");
            }

            report.entries.push(ReportEntry { name, score, time });
        }
    }

    Ok(report)
}

fn parse_filename(filename: &std::ffi::OsString) -> anyhow::Result<(String, usize, u64, bool)> {
    static NAME_ERR: &str = "Cannot read filename";

    let name = filename.to_str().ok_or_else(|| anyhow!(NAME_ERR))?;
    let mut parts = name.split('.');
    let mut parts = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.split('_');
    let machines = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.parse()?;
    let result = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.parse()?;
    let _: usize = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.parse()?;
    let is_unit = parts.next().is_some_and(|x| x == "unit");
    Ok((name.into(), machines, result, is_unit))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_filename() -> anyhow::Result<()> {
        let filename = "10_1234_0_unit.in".into();
        let (name, machines, result, is_unit) = parse_filename(&filename)?;
        assert_eq!(name, "10_1234_0_unit.in");
        assert_eq!(machines, 10);
        assert_eq!(result, 1234);
        assert!(is_unit);

        let filename = "2_14_2.in".into();
        let (name, machines, result, is_unit) = parse_filename(&filename)?;
        assert_eq!(name, "2_14_2.in");
        assert_eq!(machines, 2);
        assert_eq!(result, 14);
        assert!(!is_unit);
        Ok(())
    }

    #[test]
    fn test_parse_filename_errors() {
        assert!(parse_filename(&"".into()).is_err());
        assert!(parse_filename(&".in".into()).is_err());
        assert!(parse_filename(&"10.in".into()).is_err());
        assert!(parse_filename(&"10_1234.in".into()).is_err());
        assert!(parse_filename(&"10_1a234_0_unit.in".into()).is_err());
        assert!(parse_filename(&"1a0_1234_0.in".into()).is_err());
        assert!(parse_filename(&"10_1234_0a2.in".into()).is_err());
    }
}
