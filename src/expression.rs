use crate::parsing::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Expression {
    Number(f64),
    Variable(String),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Divide(Box<Expression>, Box<Expression>),
    Power(Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn evaluate(&self, variables: &HashMap<String, f64>) -> Result<f64, String> {
        match self {
            Expression::Number(n) => Ok(*n),
            Expression::Variable(name) => variables
                .get(name)
                .copied()
                .ok_or(format!("Variable '{}' not found", name)),
            Expression::Add(a, b) => Ok(a.evaluate(variables)? + b.evaluate(variables)?),
            Expression::Subtract(a, b) => Ok(a.evaluate(variables)? - b.evaluate(variables)?),
            Expression::Multiply(a, b) => Ok(a.evaluate(variables)? * b.evaluate(variables)?),
            Expression::Divide(a, b) => {
                let denominator = b.evaluate(variables)?;
                if denominator == 0.0 {
                    return Err("Division by 0".to_string());
                }
                Ok(a.evaluate(variables)? / denominator)
            }
            Expression::Power(base, exponent) => Ok(base
                .evaluate(variables)?
                .powf(exponent.evaluate(variables)?)),
        }
    }

    pub fn parse(input: &str) -> Result<Expression, String> {
        Parser::new(tokenize(input)?).parse_expression()
    }
}

pub mod expr {
    use super::Expression;

    pub fn number(n: f64) -> Expression {
        Expression::Number(n)
    }

    pub fn variable(name: &str) -> Expression {
        Expression::Variable(name.to_string())
    }

    pub fn add(a: Expression, b: Expression) -> Expression {
        Expression::Add(Box::new(a), Box::new(b))
    }

    pub fn subtract(a: Expression, b: Expression) -> Expression {
        Expression::Subtract(Box::new(a), Box::new(b))
    }

    pub fn multiply(a: Expression, b: Expression) -> Expression {
        Expression::Multiply(Box::new(a), Box::new(b))
    }

    pub fn divide(a: Expression, b: Expression) -> Expression {
        Expression::Divide(Box::new(a), Box::new(b))
    }

    pub fn power(base: Expression, power: Expression) -> Expression {
        Expression::Power(Box::new(base), Box::new(power))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use expression_macro::expr;
    use std::collections::HashMap;

    // Helper function to create variables map
    fn create_vars() -> HashMap<String, f64> {
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), 2.0);
        vars.insert("y".to_string(), 3.0);
        vars
    }

    #[test]
    fn test_basic_operations() {
        let vars = create_vars();

        assert_eq!(expr!("2 + 3").evaluate(&vars).unwrap(), 5.0);
        assert_eq!(expr!("5 - 3").evaluate(&vars).unwrap(), 2.0);
        assert_eq!(expr!("4 * 2").evaluate(&vars).unwrap(), 8.0);
        assert_eq!(expr!("8 / 2").evaluate(&vars).unwrap(), 4.0);
        assert_eq!(expr!("2 ^ 3").evaluate(&vars).unwrap(), 8.0);
    }

    #[test]
    fn test_variable_operations() {
        let vars = create_vars();

        assert_eq!(expr!("x + y").evaluate(&vars).unwrap(), 5.0);
        assert_eq!(expr!("x * y").evaluate(&vars).unwrap(), 6.0);
        assert_eq!(expr!("y - x").evaluate(&vars).unwrap(), 1.0);
        assert_eq!(expr!("y / x").evaluate(&vars).unwrap(), 1.5);
        assert_eq!(expr!("x ^ 2").evaluate(&vars).unwrap(), 4.0);
    }

    #[test]
    fn test_complex_expressions() {
        let vars = create_vars();

        assert_eq!(expr!("(x + 1) ^ 2").evaluate(&vars).unwrap(), 9.0);
        assert_eq!(expr!("2 * x + y").evaluate(&vars).unwrap(), 7.0);
        assert_eq!(expr!("(x + y) * 2").evaluate(&vars).unwrap(), 10.0);
        assert_eq!(expr!("x ^ 2 + y ^ 2").evaluate(&vars).unwrap(), 13.0);
    }

    #[test]
    fn test_operator_precedence() {
        let vars = create_vars();

        assert_eq!(expr!("2 + 3 * 4").evaluate(&vars).unwrap(), 14.0);
        assert_eq!(expr!("(2 + 3) * 4").evaluate(&vars).unwrap(), 20.0);
        assert_eq!(expr!("2 ^ 2 * 3").evaluate(&vars).unwrap(), 12.0);
        assert_eq!(expr!("2 * 3 ^ 2").evaluate(&vars).unwrap(), 18.0);
    }

    #[test]
    fn test_error_handling() {
        let vars = create_vars();
        let empty_vars = HashMap::new();

        // Division by zero
        assert!(expr!("x / 0").evaluate(&vars).is_err());

        // Unknown variable
        assert!(expr!("z + 1").evaluate(&vars).is_err());

        // Missing variable
        assert!(expr!("x + y").evaluate(&empty_vars).is_err());
    }

    #[test]
    fn test_whitespace_handling() {
        let vars = create_vars();

        assert_eq!(expr!("x+y").evaluate(&vars).unwrap(), 5.0);
        assert_eq!(expr!("x + y").evaluate(&vars).unwrap(), 5.0);
        assert_eq!(expr!(" x  +  y ").evaluate(&vars).unwrap(), 5.0);
    }

    #[test]
    fn test_nested_expressions() {
        let vars = create_vars();

        assert_eq!(expr!("((x + 1) * (y - 1))").evaluate(&vars).unwrap(), 6.0);
        assert_eq!(expr!("(x + y) * (x - y)").evaluate(&vars).unwrap(), -5.0);
    }

    #[test]
    fn test_builder_api() {
        let vars = create_vars();

        let expr = expr::power(
            expr::add(expr::variable("x"), expr::number(1.0)),
            expr::number(2.0),
        );
        assert_eq!(expr.evaluate(&vars).unwrap(), 9.0);
    }

    #[test]
    fn test_floating_point_numbers() {
        let vars = create_vars();

        assert_eq!(expr!("2.5 + 1.5").evaluate(&vars).unwrap(), 4.0);
        assert_eq!(expr!("3.14159 * 2").evaluate(&vars).unwrap(), 6.28318);
    }
}
