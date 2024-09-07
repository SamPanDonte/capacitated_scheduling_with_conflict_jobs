#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
use super::gurobi::{conflict_vars, create_model, tardy_vars};
use crate::core::{Instance, Machine, Schedule, ScheduleInfo, Scheduler, Task};
use crate::{cast_u64, cast_usize};
use anyhow::Result;
use grb::prelude::*;
use std::collections::BTreeSet;

/// ILP2 scheduler.
/// This scheduler uses the Gurobi solver to solve the instance.
/// Its solve function panics if the Gurobi solver fails.
#[derive(Clone, Debug, Default)]
pub struct ILP2;

impl Scheduler for ILP2 {
    fn schedule<'a>(&mut self, instance: &'a Instance) -> Schedule<'a> {
        ilp2_impl(instance).unwrap_or_else(|err| panic!("Gurobi failed {err}"))
    }

    fn name(&self) -> &'static str {
        "ILP2"
    }
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(super::SCHEDULERS)]
static INSTANCE: fn() -> Box<dyn Scheduler> = || Box::new(ILP2);

#[allow(clippy::useless_conversion)]
fn ilp2_impl(instance: &Instance) -> Result<Schedule> {
    if instance.tasks.is_empty() {
        return Ok(Schedule::new(instance));
    }

    let mut model = create_model("ILP2")?;

    let tasks = &instance.tasks;
    let d = cast_usize(instance.deadline);

    let u = tardy_vars(&mut model, tasks.len())?;
    let y = conflict_vars(&mut model, instance)?;
    let v = position_vars(&mut model, tasks, d)?;

    for (j, (&uj, vj)) in u.iter().zip(v.iter()).enumerate() {
        model.add_constr(&format!("c_0_{j}"), c!(vj.iter().grb_sum() + uj == 1))?;
    }

    for t in 0..d {
        let iter = v.iter().zip(tasks);
        let expr = iter.map(|(vj, task)| c1_sum(d, t, vj, task)).grb_sum();
        model.add_constr(&format!("c_1_{t}"), c!(expr <= instance.processors))?;
    }

    for (j, vars) in y.iter().enumerate() {
        for (&g, &var) in vars {
            let pj = cast_usize(tasks[j].time);
            let pg = cast_usize(tasks[g].time);

            let iter = v[j][0..=d - pj].iter().enumerate();
            let left = iter.map(|(t, &vjt)| t * vjt).grb_sum();
            let left = left + pj * (1 - u[j]) - d * var;

            let iter = v[g][0..=d - pg].iter().enumerate();
            let right = iter.map(|(t, &vgt)| t * vgt).grb_sum();

            model.add_constr(&format!("c_2_{j}_{g}"), c!(left <= right))?;
        }
    }

    for (j, vars) in y.iter().enumerate() {
        for (&g, &var) in vars {
            if j < g {
                model.add_constr(&format!("c_3_{j}_{g}"), c!(var + y[g][&j] <= 1))?;
            }
        }
    }

    let expr = u.iter().enumerate().map(|(j, &uj)| uj * tasks[j].weight);
    model.set_objective(expr.grb_sum(), Minimize)?;
    model.optimize()?;

    let mut result = Schedule::new(instance);
    let mut machines: BTreeSet<_> = (0..instance.processors).map(Machine::new).collect();

    for t in 0..d {
        for (j, task) in v.iter().enumerate() {
            if task.len() > t && model.get_obj_attr(attr::X, &task[t])? as i64 == 1 {
                let Some(machine) = machines.iter().find(|m| m.free <= cast_u64(t)) else {
                    unreachable!("Must be free machine before time `t`");
                };

                result.schedule(j, ScheduleInfo::new(cast_u64(t), machine.id));

                let mut machine = *machine;
                machines.remove(&machine);
                machine.free = cast_u64(t) + tasks[j].time;
                machines.insert(machine);
            }
        }
    }

    Ok(result)
}

fn position_vars(model: &mut Model, tasks: &[Task], d: usize) -> Result<Vec<Vec<Var>>> {
    let mut w = vec![Vec::new(); tasks.len()];
    for ((j, wj), task) in w.iter_mut().enumerate().zip(tasks) {
        for t in 0..=d - cast_usize(task.time) {
            wj.push(add_binvar!(model, name: &format!("v_{j}_{t}"))?);
        }
    }
    Ok(w)
}

fn c1_sum(d: usize, t: usize, vj: &[Var], task: &Task) -> Expr {
    let pj = cast_usize(task.time);
    let s = if t < pj { 0 } else { t + 1 - pj };
    vj[s..=t.min(d - cast_usize(task.time))].iter().grb_sum()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::data::samples;

    #[test]
    fn test_ilp2() {
        assert!(samples(usize::MAX, &mut ILP2).is_ok());
    }
}
