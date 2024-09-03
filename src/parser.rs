use crate::grammar::*;

pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = vec![];
        while !self.end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Statement, String> {
        if self.match_(&[TokenType::VAR]) {
            return self.variable();
        } else if self.match_(&[TokenType::PRINT]) {
            let expression = self.expression()?;
            self.consume(&TokenType::SEMICOLON, "Expect ';' after value.")?;
            Ok(Statement::Print(expression))
        } else if self.match_(&[TokenType::LEFT_BRACE]) {
            let mut statements = vec![];
            while !self.is_cur_match(&TokenType::RIGHT_BRACE) && !self.end() {
                statements.push(self.statement()?);
            }
            self.consume(&TokenType::RIGHT_BRACE, "Expect '}' after block.")?;
            Ok(Statement::Block(statements))
        } else {
            let expression = self.expression()?;
            self.consume(&TokenType::SEMICOLON, "Expect ';' after expression.")?;
            Ok(Statement::Expression(expression))
        }
    }

    fn variable(&mut self) -> Result<Statement, String> {
        let name = self
            .consume(&TokenType::IDENTIFIER, "Expect variable name.")?
            .clone();
        let init = if self.match_(&[TokenType::EQUAL]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(
            &TokenType::SEMICOLON,
            "Expect ';' after variable declaration.",
        )?;
        Ok(Statement::Variable { name, init })
    }

    pub fn expression(&mut self) -> Result<Expression, String> {
        let expression = self.binary_operation(
            &[TokenType::BANG_EQUAL, TokenType::EQUAL_EQUAL],
            Self::comparison,
        )?;
        if self.match_(&[TokenType::EQUAL]) {
            let right = self.expression()?;
            if let Expression::Variable(name) = expression {
                return Ok(Expression::Assign {
                    name,
                    right: Box::new(right),
                });
            }
            return Err(self.error(self.previous(), "Invalid assignment target."));
        }
        Ok(expression)
    }

    fn comparison(&mut self) -> Result<Expression, String> {
        self.binary_operation(
            &[
                TokenType::GREATER,
                TokenType::GREATER_EQUAL,
                TokenType::LESS,
                TokenType::LESS_EQUAL,
            ],
            Self::term,
        )
    }

    fn term(&mut self) -> Result<Expression, String> {
        self.binary_operation(&[TokenType::MINUS, TokenType::PLUS], Self::factor)
    }

    fn factor(&mut self) -> Result<Expression, String> {
        self.binary_operation(&[TokenType::SLASH, TokenType::STAR], Self::unary)
    }

    fn binary_operation(
        &mut self,
        operators: &[TokenType],
        next_precedence: fn(&mut Self) -> Result<Expression, String>,
    ) -> Result<Expression, String> {
        let mut left = next_precedence(self)?;
        while self.match_(operators) {
            let op = self.previous().clone();
            let right = next_precedence(self)?;
            left = Expression::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    pub fn unary(&mut self) -> Result<Expression, String> {
        if self.match_(&[TokenType::BANG, TokenType::MINUS]) {
            let op = self.previous().clone();
            let expr = self.unary()?;
            return Ok(Expression::Unary {
                op,
                expr: Box::new(expr),
            });
        }
        self.primary()
    }

    pub fn primary(&mut self) -> Result<Expression, String> {
        if self.match_(&[TokenType::FALSE]) {
            return Ok(Expression::Literal(Literal::Boolean(false)));
        }

        if self.match_(&[TokenType::TRUE]) {
            return Ok(Expression::Literal(Literal::Boolean(true)));
        }

        if self.match_(&[TokenType::NIL]) {
            return Ok(Expression::Literal(Literal::Nil));
        }

        if self.match_(&[TokenType::NUMBER, TokenType::STRING]) {
            return Ok(Expression::Literal(
                self.previous().literal.as_ref().unwrap().clone(),
            ));
        }

        if self.match_(&[TokenType::IDENTIFIER]) {
            return Ok(Expression::Variable(self.previous().clone()));
        }

        if self.match_(&[TokenType::LEFT_PAREN]) {
            let expression = self.expression()?;
            self.consume(&TokenType::RIGHT_PAREN, "Expect ')' after expression.")?;
            return Ok(Expression::Group(Box::new(expression)));
        }

        Err(self.error(self.peek(), "Expect expression."))
    }

    fn match_(&mut self, token_types: &[TokenType]) -> bool {
        let is_match = token_types
            .iter()
            .any(|token_type| self.is_cur_match(token_type));
        if is_match {
            self.advance();
        }
        is_match
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token, String> {
        if self.is_cur_match(token_type) {
            return Ok(self.advance());
        }
        Err(self.error(self.peek(), message))
    }

    fn is_cur_match(&self, token_type: &TokenType) -> bool {
        !self.end() && self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.end() {
            self.current += 1;
        }
        self.previous()
    }

    fn end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn error(&self, token: &Token, message: &str) -> String {
        format!(
            "[line {}] Error at '{}': {}",
            token.line_num, token.lexeme, message
        )
    }
}
