use crate::core::Instance;
use ahash::{HashMap, HashMapExt};
use anyhow::Result;
use grb::{add_binvar, param, Env, Model, Var};

pub fn create_model(name: &str) -> Result<Model> {
    let mut env = Env::new("")?;
    env.set(param::OutputFlag, 0)?;
    env.set(param::LogToConsole, 0)?;
    env.set(param::TimeLimit, 3600.0)?;
    Ok(Model::with_env(name, env)?)
}

pub fn tardy_vars(model: &mut Model, n: usize) -> Result<Vec<Var>> {
    let mut u = Vec::with_capacity(n);
    for j in 0..n {
        u.push(add_binvar!(model, name: &format!("u_{j}"))?);
    }
    Ok(u)
}

pub fn conflict_vars(model: &mut Model, instance: &Instance) -> Result<Vec<HashMap<usize, Var>>> {
    let mut y = vec![HashMap::new(); instance.tasks.len()];
    for (j, yj) in y.iter_mut().enumerate() {
        for &g in instance.graph.conflicts(j) {
            yj.insert(g, add_binvar!(model, name: &format!("y_{j}_{g}"))?);
        }
    }
    Ok(y)
}
