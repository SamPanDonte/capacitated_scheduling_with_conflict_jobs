#[cfg(feature = "gurobi")]
mod gurobi;
mod list;
mod matching;
mod polynomial_time;
mod tresoldi;
mod vns;

#[cfg(feature = "gurobi")]
pub use gurobi::gurobi;
pub use list::list;
pub use polynomial_time::polynomial_time;
pub use tresoldi::Tresoldi;
pub use vns::VariableNeighborhoodSearch;

use crate::core::Scheduler;
use crate::data::deserialize;
use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::BufReader;

static NAME_ERR: &str = "Cannot read filename";

/// Run all samples in the `samples` directory.
///
/// # Arguments
/// - `unit` is true, only run the unit samples.
/// - `score` is true, check if the score is correct.
/// - `m` is the maximum number of machines to run.
/// - `solver` is the scheduler to run.
///
/// # Errors
///  - If a file cannot be read.
///
/// # Panics
/// - If the schedule is invalid.
/// - If the score is incorrect.
pub fn run_samples(unit: bool, score: bool, m: usize, solver: &impl Scheduler) -> Result<()> {
    for file in std::fs::read_dir("samples")? {
        let file = file?;
        let filename = file.file_name();
        let (name, machines, result, is_unit) = parse_filename(&filename)?;

        if (!unit || is_unit) && machines <= m {
            let instance = deserialize(&mut BufReader::new(File::open(file.path())?))?;
            let schedule = solver.clone().schedule(&instance);
            assert!(schedule.verify(), "Invalid schedule created");
            if score {
                assert_eq!(schedule.calculate_score(), result, "Invalid score {name}");
            }

            println!("{name}: OK");
        }
    }

    Ok(())
}

fn parse_filename(filename: &std::ffi::OsString) -> Result<(&str, usize, u64, bool)> {
    let name = filename.to_str().ok_or_else(|| anyhow!(NAME_ERR))?;
    let mut parts = name.split('.');
    let mut parts = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.split('_');
    let machines = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.parse()?;
    let result = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.parse()?;
    let _: usize = parts.next().ok_or_else(|| anyhow!(NAME_ERR))?.parse()?;
    let is_unit = parts.next().is_some_and(|x| x == "unit");
    Ok((name, machines, result, is_unit))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_filename() -> Result<()> {
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
