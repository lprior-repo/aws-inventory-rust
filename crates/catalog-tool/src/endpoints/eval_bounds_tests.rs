use super::{Bindings, expression};
use crate::endpoints::{Fault, MAX_DEPTH, MAX_STEPS, model::Expr};
use serde_json::Value;

#[test]
fn evaluation_depth_exactly_64_succeeds_and_65_fails() {
    let expression_value = Expr::Literal(Value::Bool(true));
    let env = Bindings::new();
    let mut steps = MAX_STEPS;
    assert_eq!(
        expression(&expression_value, &env, MAX_DEPTH, &mut steps),
        Ok(Value::Bool(true))
    );
    assert_eq!(
        expression(
            &expression_value,
            &env,
            MAX_DEPTH.saturating_add(1),
            &mut steps
        ),
        Err(Fault::Bound("depth"))
    );
}

#[test]
fn evaluation_accepts_exactly_100000_steps_and_rejects_the_next() {
    let expression_value = Expr::Literal(Value::Bool(true));
    let env = Bindings::new();
    let mut steps = MAX_STEPS;
    for _ in 0..MAX_STEPS {
        assert_eq!(
            expression(&expression_value, &env, 0, &mut steps),
            Ok(Value::Bool(true))
        );
    }
    assert_eq!(steps, 0);
    assert_eq!(
        expression(&expression_value, &env, 0, &mut steps),
        Err(Fault::Bound("steps"))
    );
}
