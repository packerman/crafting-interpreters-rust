use super::{
    error::ErrorReporter,
    expr::Expr,
    token::{Token, TokenKind},
};

pub struct Parser<'a> {
    tokens: Vec<Token>,
    current: usize,
    error_reporter: &'a ErrorReporter,
}

impl<'a> Parser<'a> {
    const EQUALITY_OPERATORS: [TokenKind; 2] = [TokenKind::BangEqual, TokenKind::EqualEqual];
    const COMPARISON_OPERATORS: [TokenKind; 4] = [
        TokenKind::Greater,
        TokenKind::GreaterEqual,
        TokenKind::Less,
        TokenKind::LessEqual,
    ];
    const TERM_OPERATORS: [TokenKind; 2] = [TokenKind::Minus, TokenKind::Plus];
    const FACTOR_OPERATORS: [TokenKind; 2] = [TokenKind::Slash, TokenKind::Star];
    const UNARY_OPERATORS: [TokenKind; 2] = [TokenKind::Bang, TokenKind::Minus];
    const SYNCHRONIZE: [TokenKind; 8] = [
        TokenKind::Class,
        TokenKind::Fun,
        TokenKind::Var,
        TokenKind::For,
        TokenKind::If,
        TokenKind::While,
        TokenKind::Print,
        TokenKind::Return,
    ];

    pub fn new(tokens: Vec<Token>, error_reporter: &'a ErrorReporter) -> Self {
        Self {
            tokens,
            current: 0,
            error_reporter,
        }
    }

    pub fn parse(&mut self) -> Option<Box<Expr>> {
        self.expression()
    }

    fn expression(&mut self) -> Option<Box<Expr>> {
        self.ternary()
    }

    fn ternary(&mut self) -> Option<Box<Expr>> {
        let expr = self.equality()?;
        if self.match_single(&TokenKind::QuestionMark) {
            let then_expr = self.expression()?;
            self.consume(&TokenKind::Colon, || "Expect ':'.".into());
            let else_expr = self.expression()?;
            Some(Box::new(Expr::Ternary(expr, then_expr, else_expr)))
        } else {
            Some(expr)
        }
    }

    fn equality(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::EQUALITY_OPERATORS, Self::comparison)
    }

    fn comparison(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::COMPARISON_OPERATORS, Self::term)
    }

    fn term(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::TERM_OPERATORS, Self::factor)
    }

    fn factor(&mut self) -> Option<Box<Expr>> {
        self.binary(&Self::FACTOR_OPERATORS, Self::unary)
    }

    fn binary<F>(&mut self, operators: &[TokenKind], mut operand: F) -> Option<Box<Expr>>
    where
        F: FnMut(&mut Self) -> Option<Box<Expr>>,
    {
        let mut expr = operand(self)?;
        while self.match_any(operators) {
            expr = Box::new(Expr::Binary(
                expr,
                self.previous().to_owned(),
                operand(self)?,
            ));
        }
        Some(expr)
    }

    fn unary(&mut self) -> Option<Box<Expr>> {
        if self.match_any(&Self::UNARY_OPERATORS) {
            let operator = self.previous().to_owned();
            let right = self.unary()?;
            Some(Box::new(Expr::Unary(operator, right)))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Option<Box<Expr>> {
        let expr = if self.match_single(&TokenKind::False) {
            Expr::from(false)
        } else if self.match_single(&TokenKind::True) {
            Expr::from(true)
        } else if self.match_single(&TokenKind::Nil) {
            Expr::from(())
        } else if let Some(literal) = self.literal() {
            literal
        } else if self.match_single(&TokenKind::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenKind::RightParen, || {
                "Expect ')' after expression.".into()
            })?;
            Expr::Grouping(expr)
        } else {
            self.error(self.peek(), "Expect expression")?
        };
        Some(Box::new(expr))
    }

    fn literal(&mut self) -> Option<Expr> {
        let expr = if self.is_at_end() {
            None
        } else if let TokenKind::Number(number) = self.peek().kind {
            Some(Expr::from(number))
        } else if let TokenKind::String(string) = &self.peek().kind {
            Some(Expr::from(string.as_str()))
        } else {
            None
        };
        if expr.is_some() {
            self.advance();
        }
        expr
    }

    fn match_any(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn match_single(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume<M>(&mut self, kind: &TokenKind, message: M) -> Option<&Token>
    where
        M: Fn() -> String,
    {
        if self.check(kind) {
            Some(self.advance())
        } else {
            self.error(self.peek(), &message())
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().kind == kind
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error<T>(&self, token: &Token, message: &str) -> Option<T> {
        self.error_reporter.token_error(token, message);
        None
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().kind == TokenKind::Semicolon {
                return;
            }
            if Self::SYNCHRONIZE.contains(&self.peek().kind) {
                return;
            }
            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::walk_tree::{error::ErrorReporter, scanner::Scanner};

    use super::*;

    #[test]
    fn parsing_literals_works() {
        assert_eq!(test_parse_expr("2").unwrap().as_ref(), &Expr::from(2.0));
        assert_eq!(test_parse_expr("true").unwrap().as_ref(), &Expr::from(true));
        assert_eq!(
            test_parse_expr("false").unwrap().as_ref(),
            &Expr::from(false)
        );
        assert_eq!(test_parse_expr("nil").unwrap().as_ref(), &Expr::from(()));
        assert_eq!(
            test_parse_expr("\"abc\"").unwrap().as_ref(),
            &Expr::from("abc")
        );
    }

    #[test]
    fn parsing_expressions_works() {
        assert_eq!(
            test_parse_expr("2+2").unwrap().as_ref(),
            &Expr::Binary(
                Box::new(Expr::from(2.0)),
                Token::new(TokenKind::Plus, "+".into(), 1),
                Box::new(Expr::from(2.0))
            )
        );
        assert_eq!(
            test_parse_expr("1+2*3").unwrap().as_ref(),
            &Expr::Binary(
                Box::new(Expr::from(1.0)),
                Token::new(TokenKind::Plus, "+".into(), 1),
                Box::new(Expr::Binary(
                    Box::new(Expr::from(2.0)),
                    Token::new(TokenKind::Star, "*".into(), 1),
                    Box::new(Expr::from(3.0))
                ))
            )
        );
        assert_eq!(
            test_parse_expr("(1+2)*3").unwrap().as_ref(),
            &Expr::Binary(
                Box::new(Expr::Grouping(Box::new(Expr::Binary(
                    Box::new(Expr::from(1.0)),
                    Token::new(TokenKind::Plus, "+".into(), 1),
                    Box::new(Expr::from(2.0))
                )))),
                Token::new(TokenKind::Star, "*".into(), 1),
                Box::new(Expr::from(3.0))
            )
        );
        assert_eq!(
            test_parse_expr("1 + 2 + 3").unwrap().as_ref(),
            &Expr::Binary(
                Box::new(Expr::Binary(
                    Box::new(Expr::from(1.0)),
                    Token::new(TokenKind::Plus, "+".into(), 1),
                    Box::new(Expr::from(2.0))
                )),
                Token::new(TokenKind::Plus, "+".into(), 1),
                Box::new(Expr::from(3.0)),
            )
        );
    }

    #[test]
    fn parsing_comperison_works() {
        assert_eq!(
            test_parse_expr("2 < 3").unwrap().as_ref(),
            &Expr::Binary(
                Box::new(Expr::from(2.0)),
                Token::new(TokenKind::Less, "<".into(), 1),
                Box::new(Expr::from(3.0))
            )
        );
    }

    #[test]
    fn parsing_ternary_works() {
        assert_eq!(
            test_parse_expr("2 < 3 ? 4 : 5").unwrap().as_ref(),
            &Expr::Ternary(
                Box::new(Expr::Binary(
                    Box::new(Expr::from(2.0)),
                    Token::new(TokenKind::Less, "<".into(), 1),
                    Box::new(Expr::from(3.0))
                )),
                Box::new(Expr::from(4.0)),
                Box::new(Expr::from(5.0))
            )
        );
    }

    fn test_parse_expr(source: &str) -> Option<Box<Expr>> {
        let error_reporer = ErrorReporter::new();
        let scanner = Scanner::new(&error_reporer);
        let tokens: Vec<_> = scanner.scan_tokens(source).collect();
        let mut parser = Parser::new(tokens, &error_reporer);
        parser.parse()
    }
}
