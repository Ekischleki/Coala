use crate::{code_location::CodeLocation, compilation::Compilation, token::{Brace, BraceState, Delimiter, Token, TokenType}, type_stream::TypeStream};

#[derive(Debug)]
pub enum TokenBlock {
    Token(Token),
    Block(Block)
}

#[derive(Debug)]
pub struct Block {
    pub span: CodeLocation,
    pub brace_type: Brace,
    pub open_token: Token,
    pub body: Vec<TokenBlock>,
    pub close_token: Option<Token>
}

impl TokenBlock {
    pub fn code_location(&self) -> &CodeLocation {
        match self {
            TokenBlock::Block(b) => &b.span,
            TokenBlock::Token(t) => t.code_location()
        }
    }
    pub fn token_type(&self) -> TokenBlockType {
        match self {
            TokenBlock::Token(t) => TokenBlockType::Token(t.token_type()),
            TokenBlock::Block(Block { brace_type, .. }) => TokenBlockType::Block(brace_type)
        }
    }
    pub fn into_token_or_none(self) -> (CodeLocation, Option<Token>) {
        match self {
            TokenBlock::Token(t) => (t.code_location().clone(), Some(t)),
            TokenBlock::Block(b) => (b.span, None)
        }
    }
    pub fn as_token_or_none(&self) -> (CodeLocation, Option<&Token>) {
        match self {
            TokenBlock::Token(t) => (t.code_location().clone(), Some(t)),
            TokenBlock::Block(b) => (b.span.clone(), None)
        }
    }
    pub fn into_block_or_none(self) -> (CodeLocation, Option<Block>) {
        match self {
            TokenBlock::Token(t) => (t.code_location().to_owned(), None),
            TokenBlock::Block(b) => (b.span.to_owned(), Some(b))
        }
    }
    pub fn into_block_type_or_error(self, compilation: &mut Compilation, block_type: Brace) -> Option<Block> {
        let (location, block) = self.into_block_or_none();
        if let Some(b) = block {
            if b.brace_type == block_type{
                return Some(b);
            }
        }
        compilation.add_error(&format!("Expected {:?} block", block_type), Some(location));
        return None;
    }
    pub fn as_block_or_none(&self) -> (CodeLocation, Option<&Block>) {
        match self {
            TokenBlock::Token(t) => (t.code_location().to_owned(), None),
            TokenBlock::Block(b) => (b.span.to_owned(), Some(b))
        }
    }
    pub fn into_identifier_or_error(self, compilation: &mut Compilation) -> Option<String> {
        let (location, token) = self.into_token_or_none();
        if let Some(token) = token {
            if let Ok(identifier) = token.into_token_type().into_identifier() {
                return Some(identifier);
            }
        }
        compilation.add_error("Expected identifier", Some(location));
        return None;
    }

    pub fn assert_is_delimiter_or_error(&self, compilation: &mut Compilation, expected_delimiter: Delimiter) -> Option<()> {
        let (location, token) = self.as_token_or_none();
        if let Some(token) = token {
            if let Some(found_delim) = token.token_type().as_delimiter() {
                if found_delim == &expected_delimiter {
                    return Some(())
                } else {
                    return None;
                }
            }
        }
        compilation.add_error(&format!("Expected delimiter: {:?}", expected_delimiter), Some(location));
        return None;
    }
}
#[derive(PartialEq, Eq)]
pub enum TokenBlockType<'a> {
    Token(&'a TokenType),
    Block(&'a Brace)
}

impl TokenBlockType<'_> {
    pub fn as_delimiter(&self) -> Option<&Delimiter> {
        match self {
            Self::Token(t) => {
                t.as_delimiter()
            }
            Self::Block(_b) => {
                None
            }
        }
    }
    pub fn is_double_colon(&self) -> bool {
        self.as_delimiter().map(|d| d.is_double_colon()).is_some_and(|s| s)
    }
    pub fn is_period(&self) -> bool {
        self.as_delimiter().map(|d| d.is_period()).is_some_and(|s| s)
    }
}

impl TokenBlock {
    pub fn from_token_stream(mut token_stream: TypeStream<Token>, compilation:  &mut Compilation) -> Result<TypeStream<Self>, ()> {
        let mut res = vec![];
        loop {
            let token = token_stream.next();
            match token.token_type() {
                TokenType::Delimiter(Delimiter::Brace(open_brace_type, BraceState::Open)) => {
                    let open_brace_type = *open_brace_type;
                    res.push(Self::parse_block(&mut token_stream, compilation, open_brace_type, token).ok_or(())?);
                }
                TokenType::Delimiter(Delimiter::Brace(brace_type, BraceState::Closed)) => {
                    compilation.add_error(&format!("Unexpected {:?}", brace_type), Some(token.code_location().to_owned()));
                    return Err(())
                }
                TokenType::EOF => break,
                _ => {
                    res.push(TokenBlock::Token(token));
                }
            }
        }

        Ok(TypeStream::from_iter(res.into_iter()))

    }

    fn parse_block(token_stream: &mut TypeStream<Token>, compilation:  &mut Compilation, exit_brace_type: Brace, open_token: Token) -> Option<Self> {
        let mut body = vec![];

        loop {
            let token = token_stream.peek()?;
            match token.token_type() {
                TokenType::Delimiter(Delimiter::Brace(closed_brace_type, BraceState::Closed)) => {
                    let closed_brace_type = *closed_brace_type;

                    let close_token = token_stream.next();
                    if closed_brace_type == exit_brace_type {
                        return Some(TokenBlock::Block(Block{ brace_type: exit_brace_type, span: open_token.to(&close_token), open_token, body, close_token: Some(close_token) }))
                    } else {
                        compilation.add_error(&format!("Expected {:?}", exit_brace_type), Some(close_token.code_location().to_owned()));
                        return None; 
                        //return Some(TokenBlock::Block { brace_type: exit_brace_type, open_token, body, close_token: Some(close_token) })
                    }
                }

                TokenType::EOF => {
                    compilation.add_error(&format!("Expected {:?}", exit_brace_type), Some(token.code_location().to_owned()));
                    return None
                }

                TokenType::Delimiter(Delimiter::Brace(open_brace_type, BraceState::Open)) => {
                    let open_brace_type = *open_brace_type;
                    let open_token = token_stream.next();
                    body.push(Self::parse_block(token_stream, compilation, open_brace_type, open_token)?);
                }

                _ => {
                    body.push(TokenBlock::Token(token_stream.next()));
                }
                
            }
        }
    }
}