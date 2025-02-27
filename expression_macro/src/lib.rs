use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{LitStr, parse_macro_input};

#[proc_macro]
pub fn expr(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let expr_str = input.value();

    let tokens = match tokenize(&expr_str) {
        Ok(t) => t,
        Err(e) => {
            return syn::Error::new_spanned(&input, e).to_compile_error().into(); // This makes the error compile time
        }
    };

    let mut parser = Parser::new(tokens);
    let expr = match parser.parse_expression() {
        Ok(expr) => expr,
        Err(e) => {
            return syn::Error::new_spanned(&input, e).to_compile_error().into();
        }
    };

    expr.to_token_stream().into()
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Variable(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
}

enum Expr {
    Number(f64),
    Variable(String),
    Add(Box<Expr>, Box<Expr>),
    Subtract(Box<Expr>, Box<Expr>),
    Multiply(Box<Expr>, Box<Expr>),
    Divide(Box<Expr>, Box<Expr>),
    Power(Box<Expr>, Box<Expr>),
}

/// This generates the AST
impl ToTokens for Expr {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Expr::Number(n) => {
                tokens.extend(quote! { Expression::Number(#n) });
            }
            Expr::Variable(name) => {
                tokens.extend(quote! { Expression::Variable(#name.to_string()) });
            }
            Expr::Add(a, b) => {
                tokens.extend(quote! { Expression::Add(Box::new(#a), Box::new(#b)) });
            }
            Expr::Subtract(a, b) => {
                tokens.extend(quote! { Expression::Subtract(Box::new(#a), Box::new(#b)) });
            }
            Expr::Multiply(a, b) => {
                tokens.extend(quote! { Expression::Multiply(Box::new(#a), Box::new(#b)) });
            }
            Expr::Divide(a, b) => {
                tokens.extend(quote! { Expression::Divide(Box::new(#a), Box::new(#b)) });
            }
            Expr::Power(a, b) => {
                tokens.extend(quote! { Expression::Power(Box::new(#a), Box::new(#b)) });
            }
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
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

    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_addition()
    }

    fn parse_addition(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_multiplication()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Plus => {
                    self.advance();
                    expr = Expr::Add(Box::new(expr), Box::new(self.parse_multiplication()?));
                }
                Token::Minus => {
                    self.advance();
                    expr = Expr::Subtract(Box::new(expr), Box::new(self.parse_multiplication()?));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_power()?;

        while let Some(token) = self.peek() {
            match token {
                Token::Star => {
                    self.advance();
                    expr = Expr::Multiply(Box::new(expr), Box::new(self.parse_power()?));
                }
                Token::Slash => {
                    self.advance();
                    expr = Expr::Divide(Box::new(expr), Box::new(self.parse_power()?));
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;

        while let Some(Token::Caret) = self.peek() {
            self.advance();
            expr = Expr::Power(Box::new(expr), Box::new(self.parse_primary()?));
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        let token = self.advance().ok_or("Unexpected end of input")?;
        match token {
            Token::Number(n) => Ok(Expr::Number(*n)),
            Token::Variable(name) => Ok(Expr::Variable(name.clone())),
            Token::LParen => {
                let expr = self.parse_expression()?;
                match self.advance() {
                    Some(Token::RParen) => Ok(expr),
                    _ => Err("Expected closing parenthesis".to_string()),
                }
            }
            _ => Err("Unexpected token".to_string()),
        }
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' | '\n' => {
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
                tokens.push(Token::Number(
                    num.parse().map_err(|_| "Invalid number format")?,
                ));
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
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '^' => {
                chars.next();
                tokens.push(Token::Caret);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            _ => return Err(format!("Unexpected character: {}", c)),
        }
    }
    Ok(tokens)
}
