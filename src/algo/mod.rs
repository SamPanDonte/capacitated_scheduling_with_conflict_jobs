#[cfg(feature = "gurobi")]
mod gurobi;
#[cfg(feature = "gurobi")]
mod ilp1;
#[cfg(feature = "gurobi")]
mod ilp2;
mod list;
mod matching;
mod polynomial_time;
mod tresoldi;
mod vns;

#[cfg(feature = "gurobi")]
pub use ilp1::ILP1;
#[cfg(feature = "gurobi")]
pub use ilp2::ILP2;
pub use list::List;
pub use polynomial_time::PolynomialTime;
pub use tresoldi::Tresoldi;
pub use vns::VariableNeighborhoodSearch;

use crate::core::Scheduler;

#[linkme::distributed_slice]
pub static SCHEDULERS: [fn() -> Box<dyn Scheduler>];
