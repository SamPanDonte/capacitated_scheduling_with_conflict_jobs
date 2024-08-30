#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
use crate::cast_usize;
use crate::core::{Instance, Schedule, ScheduleInfo, Scheduler, Task};
use ahash::{HashMap, HashMapExt};
use anyhow::Result;
use grb::prelude::*;

/// ILP1 scheduler.
/// This scheduler uses the Gurobi solver to solve the instance.
/// Its solve function panics if the Gurobi solver fails.
#[derive(Clone, Debug, Default)]
pub struct ILP1;

impl Scheduler for ILP1 {
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a> {
        ilp1_impl(instance).unwrap_or_else(|err| panic!("Gurobi failed {err}"))
    }

    fn name(&self) -> &'static str {
        "ILP1"
    }
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(super::SCHEDULERS)]
static INSTANCE: fn() -> Box<dyn Scheduler> = || Box::new(ILP1);

#[allow(clippy::useless_conversion)]
fn ilp1_impl(instance: &Instance) -> Result<Schedule> {
    if instance.tasks.is_empty() {
        return Ok(Schedule::new(instance));
    }

    let mut model = create_model("ILP1")?;

    let tasks = &instance.tasks;
    let k_max = calculate_k_max(tasks, instance.deadline);

    let w = position_vars(&mut model, tasks.len(), k_max, instance.processors)?;
    let u = tardy_vars(&mut model, tasks.len())?;
    let y = conflict_vars(&mut model, instance)?;
    let t = time_vars(&mut model, k_max, instance.processors)?;
    let tau = job_time_vars(&mut model, tasks.len())?;

    for (j, (wj, &uj)) in w.iter().zip(&u).enumerate() {
        model.add_constr(
            &format!("c_0_{j}"),
            c!(wj.iter().map(GurobiSum::grb_sum).grb_sum() + uj == 1),
        )?;
    }

    for k in 0..k_max {
        for l in 0..instance.processors {
            model.add_constr(
                &format!("c_1_{k}_{l}"),
                c!(w.iter().map(|wj| wj[k][l]).grb_sum() <= 1),
            )?;
        }
    }

    for k in 0..k_max - 1 {
        for l in 0..instance.processors {
            let expr = tasks.iter().zip(&w).map(|(task, wj)| task.time * wj[k][l]);
            model.add_constr(
                &format!("c_2_{k}_{l}"),
                c!(t[k][l] + expr.grb_sum() <= t[k + 1][l]),
            )?;
        }
    }

    for k in 0..k_max {
        for l in 0..instance.processors {
            let expr = tasks.iter().zip(&w).map(|(task, wj)| task.time * wj[k][l]);
            model.add_constr(
                &format!("c_3_{k}_{l}"),
                c!(t[k][l] + expr.grb_sum() <= instance.deadline),
            )?;
        }
    }

    for (j, &tau) in tau.iter().enumerate() {
        for (k, tk) in t.iter().enumerate() {
            for (l, &tkl) in tk.iter().enumerate() {
                model.add_constr(
                    &format!("c_4_{j}_{k}_{l}"),
                    c!(tau + instance.deadline * (1 - w[j][k][l]) >= tkl),
                )?;
            }
        }
    }

    for (j, tau) in tau.iter().enumerate() {
        for (k, tk) in t.iter().enumerate() {
            for (l, &tkl) in tk.iter().enumerate() {
                model.add_constr(
                    &format!("c_5_{j}_{k}_{l}"),
                    c!(tkl + instance.deadline * (1 - w[j][k][l]) >= tau),
                )?;
            }
        }
    }

    for (j, yj) in y.iter().enumerate() {
        for (&g, &yjg) in yj {
            model.add_constr(
                &format!("c_6_{j}_{g}"),
                c!(tau[j] + tasks[j].time * (1 - u[j]) - instance.deadline * yjg <= tau[g]),
            )?;
        }
    }

    for (j, vars) in y.iter().enumerate() {
        for (&g, &var) in vars {
            model.add_constr(&format!("c_7_{j}_{g}"), c!(var + y[g][&j] <= 1))?;
        }
    }

    let expr = u.iter().enumerate().map(|(j, &uj)| uj * tasks[j].weight);
    model.set_objective(expr.grb_sum(), Minimize)?;
    model.optimize()?;

    let mut result = Schedule::new(instance);

    for (j, wj) in w.iter().enumerate() {
        'outer: for wjk in wj {
            for (l, var) in wjk.iter().enumerate() {
                if model.get_obj_attr(attr::X, var)? as i64 == 1 {
                    let time = model.get_obj_attr(attr::X, &tau[j])? as u64;
                    result.schedule(j, ScheduleInfo::new(time, l));
                    break 'outer;
                }
            }
        }
    }

    Ok(result)
}

fn create_model(name: &str) -> Result<Model> {
    let mut env = Env::new("")?;
    env.set(param::OutputFlag, 0)?;
    env.set(param::LogToConsole, 0)?;
    Ok(Model::with_env(name, env)?)
}

fn calculate_k_max(tasks: &[Task], deadline: u64) -> usize {
    let min_time = tasks.iter().map(|task| task.time).min().unwrap_or_default();
    tasks.len().min(cast_usize(deadline / min_time))
}

fn position_vars(model: &mut Model, n: usize, k: usize, m: usize) -> Result<Vec<Vec<Vec<Var>>>> {
    let mut w = vec![vec![Vec::with_capacity(m); k]; n];
    for (j, wj) in w.iter_mut().enumerate() {
        for (k, wjk) in wj.iter_mut().enumerate() {
            for l in 0..m {
                wjk.push(add_binvar!(model, name: &format!("w_{j}_{k}_{l}"))?);
            }
        }
    }
    Ok(w)
}

fn tardy_vars(model: &mut Model, n: usize) -> Result<Vec<Var>> {
    let mut u = Vec::with_capacity(n);
    for j in 0..n {
        u.push(add_binvar!(model, name: &format!("u_{j}"))?);
    }
    Ok(u)
}

fn conflict_vars(model: &mut Model, instance: &Instance) -> Result<Vec<HashMap<usize, Var>>> {
    let mut y = vec![HashMap::new(); instance.tasks.len()];
    for (j, yj) in y.iter_mut().enumerate() {
        for &g in instance.graph.conflicts(j) {
            yj.insert(g, add_binvar!(model, name: &format!("y_{j}_{g}"))?);
        }
    }
    Ok(y)
}

fn time_vars(model: &mut Model, k: usize, m: usize) -> Result<Vec<Vec<Var>>> {
    let mut t = vec![Vec::with_capacity(m); k];
    for (k, tk) in t.iter_mut().enumerate() {
        for l in 0..m {
            tk.push(add_intvar!(model, name: &format!("t_{k}_{l}"), bounds: 0u64..)?);
        }
    }
    Ok(t)
}

fn job_time_vars(model: &mut Model, n: usize) -> Result<Vec<Var>> {
    let mut tau = Vec::with_capacity(n);
    for j in 0..n {
        tau.push(add_intvar!(model, name: &format!("p_{j}"), bounds: 0u64..)?);
    }
    Ok(tau)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::samples;

    #[test]
    fn test_gurobi() {
        assert!(samples(true, &mut ILP1).is_ok());
    }
}
