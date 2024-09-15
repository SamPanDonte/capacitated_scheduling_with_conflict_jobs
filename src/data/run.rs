use crate::core::Scheduler;
use crate::data::deserialize;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};
use std::fs::File;
use std::io::BufReader;

/// Report of running a directory of samples.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report {
    scheduler: String,
    entries: Vec<ReportEntry>,
}

impl Report {
    /// Create a new report.
    const fn new(scheduler: String) -> Self {
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

        if self.entries.is_empty() {
            writeln!(f, "No compatible samples found")?;
        }

        let mut entries = self.entries.clone();
        entries.sort_by(|a, b| parse_number(&a.name).cmp(&parse_number(&b.name)));

        #[allow(clippy::cast_precision_loss)]
        let entries_len = entries.len() as f64;
        let mut time_sum = 0.0;
        let mut error_sum = 0.0;

        for entry in entries {
            writeln!(f, "{entry}")?;
            time_sum += entry.time;
            error_sum += entry.error;
        }

        if !self.entries.is_empty() {
            let time = time_sum / entries_len;
            let error = error_sum / entries_len;

            writeln!(f, "average time {time:.2}s, average error: {error:.2}")?;
        }

        writeln!(f, "-------------------")
    }
}

/// Report of running a single sample.
#[non_exhaustive]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReportEntry {
    pub name: String,
    pub score: u64,
    pub error: f64,
    pub time: f64,
}

impl Display for ReportEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}: {:.2}s, score: {}, error: {:.2}",
            self.name, self.time, self.score, self.error
        )
    }
}

/// Run all samples in the `samples` directory.
/// Print the report to stdout.
///
/// # Arguments
/// - `valid` is the maximum number of machines to check validity,
/// - `solver` is the scheduler to run.
///
/// # Errors
/// - If a file cannot be read.
/// - If no samples are found.
///
/// # Panics
/// - If the schedule is invalid.
/// - If the score is incorrect and `score` is true.
pub fn samples(valid: usize, solver: &mut dyn Scheduler) -> anyhow::Result<()> {
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
/// - `valid` is the maximum number of machines to check validity,
/// - `solver` is the scheduler to run.
///
/// # Errors
/// - If a file cannot be read.
///
/// # Panics
/// - If the schedule is invalid.
/// - If the score is incorrect.
pub fn run(dir: &str, valid: usize, solver: &mut dyn Scheduler) -> anyhow::Result<Report> {
    let mut report = Report::new(solver.name().into());

    for file in std::fs::read_dir(dir)? {
        let file = file?;

        if file.path().extension() != Some("in".as_ref()) {
            continue;
        }

        let (name, machines, result, is_unit) = parse_filename(&file.file_name())?;

        if solver.non_unit() || is_unit {
            let instance = deserialize(&mut BufReader::new(File::open(file.path())?))?;

            let time = std::time::Instant::now();
            let schedule = solver.schedule(&instance);
            let time = time.elapsed().as_secs_f64();

            assert!(schedule.verify(), "Invalid schedule created");

            let score = schedule.calculate_score();
            if valid >= machines {
                assert_eq!(score, result, "Invalid score {name}");
            }

            #[allow(clippy::cast_precision_loss)]
            let error = 100.0 - (100 * score) as f64 / result as f64;

            report.entries.push(ReportEntry {
                name,
                score,
                error,
                time,
            });
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

fn parse_number(filename: &str) -> Option<usize> {
    filename.split('.').next()?.split('_').nth(2)?.parse().ok()
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
