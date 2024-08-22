fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rustc-link-search={}", std::env::var("GUROBI_PATH")?);
    Ok(())
}
