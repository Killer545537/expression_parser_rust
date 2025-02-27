use crate::expression::Expression;

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(f64),
    Variable(String),
    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /
    Caret,  // ^
    LParen, // (
    RParen, // )
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.current);
        self.current += 1;
        token
    }

    pub fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_multiplication()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance();
                    expr = Expression::Add(Box::new(expr), Box::new(self.parse_multiplication()?));
                }
                Token::Minus => {
                    self.advance();
                    expr = Expression::Subtract(
                        Box::new(expr),
                        Box::new(self.parse_multiplication()?),
                    );
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_power()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Star => {
                    self.advance();
                    expr = Expression::Multiply(Box::new(expr), Box::new(self.parse_power()?));
                }
                Token::Slash => {
                    self.advance();
                    expr = Expression::Divide(Box::new(expr), Box::new(self.parse_power()?));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        while let Some(Token::Caret) = self.peek() {
            self.advance();
            expr = Expression::Power(Box::new(expr), Box::new(self.parse_primary()?));
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let token = self.advance().ok_or("Unexpected end of input")?;
        match token {
            Token::Number(n) => Ok(Expression::Number(*n)),
            Token::Variable(name) => Ok(Expression::Variable(name.clone())),
            Token::LParen => {
                let expr = self.parse_expression()?;
                if self.advance() != Some(&Token::RParen) {
                    return Err("Expected closing parenthesis".to_string());
                }

                Ok(expr)
            }
            _ => Err("Unexpected token".to_string()),
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' => {
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num.parse().map_err(|_| "Invalid number")?));
            }
            'a'..='z' | 'A'..='Z' => {
                let mut name = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphabetic() {
                        name.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Variable(name));
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_numbers() {
        assert_eq!(tokenize("123.45"), Ok(vec![Token::Number(123.45)]));

        assert_eq!(tokenize("42"), Ok(vec![Token::Number(42.0)]));
    }

    #[test]
    fn test_tokenize_variables() {
        assert_eq!(
            tokenize("xyz"),
            Ok(vec![Token::Variable("xyz".to_string())])
        );

        assert_eq!(tokenize("x"), Ok(vec![Token::Variable("x".to_string())]));
    }

    #[test]
    fn test_tokenize_operators() {
        assert_eq!(
            tokenize("+-*/^"),
            Ok(vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Caret
            ])
        );
    }

    #[test]
    fn test_tokenize_parentheses() {
        assert_eq!(
            tokenize("(x)"),
            Ok(vec![
                Token::LParen,
                Token::Variable("x".to_string()),
                Token::RParen
            ])
        );
    }

    #[test]
    fn test_tokenize_complex_expression() {
        assert_eq!(
            tokenize("(x + 2.5) * y"),
            Ok(vec![
                Token::LParen,
                Token::Variable("x".to_string()),
                Token::Plus,
                Token::Number(2.5),
                Token::RParen,
                Token::Star,
                Token::Variable("y".to_string())
            ])
        );
    }

    #[test]
    fn test_tokenize_invalid_characters() {
        assert!(tokenize("x @ y").is_err());
        assert!(tokenize("2 $ 3").is_err());
        assert!(tokenize("#123").is_err());
    }

    #[test]
    fn test_tokenize_whitespace() {
        assert_eq!(tokenize("x + y"), tokenize("x+y"));

        assert_eq!(tokenize(" x  +  y "), tokenize("x+y"));
    }

    #[test]
    fn test_parse_number() {
        let tokens = vec![Token::Number(42.0)];
        let mut parser = Parser::new(tokens);

        assert_eq!(parser.parse_expression(), Ok(Expression::Number(42.0)));
    }

    #[test]
    fn test_parse_variable() {
        let tokens = vec![Token::Variable("x".to_string())];
        let mut parser = Parser::new(tokens);

        assert_eq!(
            parser.parse_expression(),
            Ok(Expression::Variable("x".to_string()))
        );
    }

    #[test]
    fn test_parse_addition() {
        let tokens = vec![Token::Number(2.0), Token::Plus, Token::Number(3.0)];
        let mut parser = Parser::new(tokens);

        assert_eq!(
            parser.parse_expression(),
            Ok(Expression::Add(
                Box::new(Expression::Number(2.0)),
                Box::new(Expression::Number(3.0))
            ))
        );
    }

    #[test]
    fn test_parse_operator_precedence() {
        let tokens = vec![
            Token::Number(2.0),
            Token::Plus,
            Token::Number(3.0),
            Token::Star,
            Token::Number(4.0),
        ];
        let mut parser = Parser::new(tokens);

        assert_eq!(
            parser.parse_expression(),
            Ok(Expression::Add(
                Box::new(Expression::Number(2.0)),
                Box::new(Expression::Multiply(
                    Box::new(Expression::Number(3.0)),
                    Box::new(Expression::Number(4.0))
                ))
            ))
        );
    }

    #[test]
    fn test_parse_parentheses() {
        let tokens = vec![
            Token::LParen,
            Token::Number(2.0),
            Token::Plus,
            Token::Number(3.0),
            Token::RParen,
            Token::Star,
            Token::Number(4.0),
        ];
        let mut parser = Parser::new(tokens);

        assert_eq!(
            parser.parse_expression(),
            Ok(Expression::Multiply(
                Box::new(Expression::Add(
                    Box::new(Expression::Number(2.0)),
                    Box::new(Expression::Number(3.0))
                )),
                Box::new(Expression::Number(4.0))
            ))
        );
    }

    #[test]
    fn test_parse_unmatched_parentheses() {
        let tokens = vec![
            Token::LParen,
            Token::Number(2.0),
            Token::Plus,
            Token::Number(3.0),
        ];
        let mut parser = Parser::new(tokens);

        assert!(parser.parse_expression().is_err());
    }

    #[test]
    fn test_parse_power() {
        let tokens = vec![Token::Number(2.0), Token::Caret, Token::Number(3.0)];
        let mut parser = Parser::new(tokens);

        assert_eq!(
            parser.parse_expression(),
            Ok(Expression::Power(
                Box::new(Expression::Number(2.0)),
                Box::new(Expression::Number(3.0))
            ))
        );
    }

    #[test]
    fn test_parse_empty() {
        let tokens = vec![];
        let mut parser = Parser::new(tokens);

        assert!(parser.parse_expression().is_err());
    }

    #[test]
    fn test_parse_incomplete_expression() {
        let tokens = vec![Token::Number(2.0), Token::Plus];
        let mut parser = Parser::new(tokens);

        assert!(parser.parse_expression().is_err());
    }
}
