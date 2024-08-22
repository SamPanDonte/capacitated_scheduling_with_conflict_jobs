use capacitated_scheduling_with_conflicts::binary_main;

#[cfg(feature = "gurobi")]
binary_main!(algo::gurobi);

#[cfg(not(feature = "gurobi"))]
fn main() {
    eprintln!("This binary requires the `gurobi` feature to be enabled.");
}
