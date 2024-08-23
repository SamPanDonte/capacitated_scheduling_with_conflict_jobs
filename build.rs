fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "gurobi")]
    println!("cargo:rustc-link-search={}", std::env::var("GUROBI_PATH")?);
    Ok(())
}
