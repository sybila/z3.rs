#![cfg(feature = "z3_4_16")]

//! Each test solves `x xor y` once biased towards each of its two models, and checks that Z3
//! returns that model. Z3 decides one atom and the other follows, so both are always assigned.
//!
//! The runs differ only in the bias, so without a working bias Z3 would return the same model
//! both times.

use z3::ast::Bool;
use z3::{Model, Optimize, Params, SatResult, Solver};

/// The models `(x, y)` of `x xor y`.
const MODELS: [(bool, bool); 2] = [(false, true), (true, false)];

/// The values of `x` and `y` in `model` (without model completion, so both must be assigned).
fn values(model: &Model, x: &Bool, y: &Bool) -> (bool, bool) {
    let value = |atom: &Bool| model.eval(atom, false).unwrap().as_bool().unwrap();
    (value(x), value(y))
}

#[test]
fn solver_follows_initial_values() {
    let (x, y) = (Bool::new_const("x"), Bool::new_const("y"));
    for (x_value, y_value) in MODELS {
        let solver = Solver::new();
        // Without a user propagator, the first check uses initial values only with this parameter.
        let mut params = Params::new();
        params.set_bool("ignore_solver1", true);
        solver.set_params(&params);

        solver.assert(x.xor(&y));
        solver.set_initial_value(&x, &Bool::from_bool(x_value));
        solver.set_initial_value(&y, &Bool::from_bool(y_value));

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        assert_eq!(values(&model, &x, &y), (x_value, y_value));
    }
}

#[test]
fn optimize_follows_initial_values() {
    let (x, y) = (Bool::new_const("x"), Bool::new_const("y"));
    for (x_value, y_value) in MODELS {
        let optimize = Optimize::new();
        optimize.assert(x.xor(&y));
        optimize.set_initial_value(&x, &Bool::from_bool(x_value));
        optimize.set_initial_value(&y, &Bool::from_bool(y_value));

        assert_eq!(optimize.check(&[]), SatResult::Sat);
        let model = optimize.get_model().unwrap();
        assert_eq!(values(&model, &x, &y), (x_value, y_value));
    }
}

#[test]
fn solver_follows_phase_bias() {
    let (x, y) = (Bool::new_const("x"), Bool::new_const("y"));
    for (x_value, y_value) in MODELS {
        let solver = Solver::new();
        // Make Z3 pick `true` at every decision, so that a `false` preference holds only if the
        // bias overrides it.
        let mut params = Params::new();
        params.set_u32("phase_selection", 1);
        solver.set_params(&params);

        solver.assert(x.xor(&y));
        solver.set_phase_bias(&[(x.clone(), x_value), (y.clone(), y_value)]);

        assert_eq!(solver.check(), SatResult::Sat);
        let model = solver.get_model().unwrap();
        assert_eq!(values(&model, &x, &y), (x_value, y_value));
    }
}

/// A clone of a biased solver has a user propagator too, and Z3 crashes on a second one.
#[test]
#[should_panic(expected = "already has a user propagator")]
fn phase_bias_of_biased_clone_panics() {
    let x = Bool::new_const("x");
    let solver = Solver::new();
    solver.set_phase_bias(&[(x.clone(), true)]);
    solver.clone().set_phase_bias(&[(x, false)]);
}
