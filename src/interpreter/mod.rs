use magc::types::*;
use magc::parser::*;
use colored::*;

use std::collections::HashMap;

pub struct Interpreter {
    /// A global namespace for variables
    environment: HashMap<String, Box<Expression>>,
    /// A special data structure which stores and linearizes multimethods.
    multimethods: HashMap<String, Multimethod>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
            multimethods: HashMap::new(),
        }
    }

    pub fn evaluate(&mut self, expr: Box<Expression>) -> Result<Box<Expression>, InterpreterError> {
        match expr.clone().kind {
            ExpressionKind::Infix(infix) => self.evaluate_infix(infix),
            ExpressionKind::Method(method) => self.evaluate_method(method),
            ExpressionKind::Literal(literal) => Ok(expr),
            ExpressionKind::Pattern(Pattern::Tuple { expr }) => Ok(self.evaluate(expr)?),

            _ => unimplemented!(),
        }
    }

    /// Add a new instance of a multimethod definition if it doesn't already exist.
    fn evaluate_method(&mut self, method: Method) -> Result<Box<Expression>, InterpreterError> {
        if let Some(mut multimethod) = self.multimethods.get_mut(&method.name) {
            // Make sure this very definition does not already exist.
            if let None = multimethod.receivers.get_mut(&method.signature) {
                multimethod.receivers.insert(
                    method.signature,
                    method.body,
                );
            } else {
                return Err(InterpreterError::MethodSignatureExists)
            }
        } else {
            self.multimethods.insert(method.name.clone(), Multimethod::new(
                method.signature,
                method.body,
            ));
        }

        println!("methods: {:#?}", self.multimethods);

        Ok(Box::new(Expression {
            kind: ExpressionKind::Identifier,
            start_pos: 0,
            end_pos: 0,
            lexeme: method.name,
        }))
    }

    fn evaluate_infix(&mut self, infix: Infix) -> Result<Box<Expression>, InterpreterError> {
        let left = self.evaluate(infix.left)?;
        let right = self.evaluate(infix.right)?;

        match infix.operator.kind {
            TokenKind::Plus => {
                Ok(Box::new(
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Float),
                        start_pos: 0,
                        end_pos: 0,
                        lexeme: format!("{}", self.extract_float(left)? + self.extract_float(right)?),
                    }
                ))
            },

            TokenKind::Star => {
                Ok(Box::new(
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Float),
                        start_pos: 0,
                        end_pos: 0,
                        lexeme: format!("{}", self.extract_float(left)? * self.extract_float(right)?),
                    }
                ))
            },

            TokenKind::Slash => {
                Ok(Box::new(
                    Expression {
                        kind: ExpressionKind::Literal(Literal::Float),
                        start_pos: 0,
                        end_pos: 0,
                        lexeme: format!("{}", self.extract_float(left)? / self.extract_float(right)?),
                    }
                ))
            },

            _ => unimplemented!(),
        }
    }

    fn extract_float(&mut self, expr: Box<Expression>) -> Result<f64, InterpreterError> {
        if let ExpressionKind::Literal(Literal::Float) = expr.kind {
            Ok(expr.lexeme.parse::<f64>().unwrap())
        } else {
            Err(InterpreterError::UnexpectedType)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InterpreterError {
    UnexpectedType,
    MethodSignatureExists,
}

/// A method definition that has one name and many pairs of function signatures and bodies.
#[derive(Debug)]
pub struct Multimethod {
    /// The individual signatures and bodies this multimethod is composed of.
    pub receivers: HashMap<Box<Expression>, Box<Expression>>,
}

impl Multimethod {
    pub fn new(signature: Box<Expression>, body: Box<Expression>) -> Self {
        let mut receivers = HashMap::new();

        receivers.insert(signature, body);

        Self {
            receivers,
        }
    }
}