use crate::{compilation::Compilation, diagnostic::{Diagnostic, DiagnosticPipelineLocation, DiagnosticType}, symbol_table::{self, ContextSymbolTable, GlobalSymbolTable}, syntax::{ArgumentSyntax, CodeSyntax, NodeValueSyntax, CollectionSyntax, SubCallSyntax, SubLocation, SubstructureSyntax, TypeSyntax}, token::{Atom, Brace, BraceState, Delimiter, Keyword, Token, TokenType}, type_stream::TypeStream};

pub struct Parser<'a> {
    tokens: TypeStream<Token>,
    compilation: &'a mut Compilation,
    symbol_table: GlobalSymbolTable
}


impl<'a> Parser<'a> {
    pub fn new(tokens: TypeStream<Token>, compilation: &'a mut Compilation, symbol_table: GlobalSymbolTable) -> Self {
        Self {
            tokens,
            compilation,
            symbol_table
        }
    }

    pub fn parse_file(&mut self) {
        while let Some(current_token) = self.tokens.peek() {
            match current_token.token_type() {
                TokenType::Keyword(Keyword::Structure) => {
                }
                _ => {panic!()}
            }
        }
    }

    fn parse_structure(&mut self) -> Option<CollectionSyntax> {
        let struct_token = self.tokens.next();
        let structure_identifier = self.tokens.next();
        
        if let TokenType::Identifier(name) = structure_identifier.into_token_type() {
            let name = name.to_owned();
        } else {
            self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected Identifier after structure keyword"), Some(struct_token.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
            return None;
        }

        let open_curly_delim = self.tokens.next();
        assert!(open_curly_delim.token_type().as_delimiter().unwrap().as_brace().unwrap().as_curly().unwrap().is_open());
        loop {
                        
        }


        todo!()
    }

    pub fn parse_sub(&mut self) -> Option<SubstructureSyntax> {
        let name = self.tokens.next().into_token_type().into_identifier().unwrap();
        assert_eq!(self.tokens.next().into_token_type(), TokenType::Delimiter(Delimiter::Brace(Brace::Round(BraceState::Open))));
        let args = self.parse_arguments()?;
        assert_eq!(self.tokens.next().into_token_type(), TokenType::Delimiter(Delimiter::Brace(Brace::Curly(BraceState::Open))));
        let code = self.parse_code_block()?;
        let mut result = None;
        if let Some(t) = self.tokens.peek() {
            if let TokenType::Delimiter(Delimiter::Equals) = t.token_type() {
                self.tokens.next();
                result = self.parse_node_value();
            }
        }
        Some(SubstructureSyntax { name, args, code, result })
    }

    pub fn parse_arguments(&mut self) -> Option<Vec<ArgumentSyntax>> {
        let mut enumeration = vec![];

        loop {
            let type_syntax = match self.parse_type() {
                Some(s) => s,
                None => {            
                    self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected type"), None, DiagnosticPipelineLocation::Parsing));
                    continue;
                }

            };
            let colon = self.tokens.next();
            assert_eq!(colon.token_type(), &TokenType::Delimiter(Delimiter::Colon));
            let name = self.tokens.next().into_token_type().into_identifier().unwrap();
            enumeration.push(ArgumentSyntax {type_syntax, name});
            let tuple_token = self.tokens.next();
            match tuple_token.token_type() {
                TokenType::Delimiter(Delimiter::Brace(Brace::Round(BraceState::Closed))) => {
                    return Some(enumeration);
                }
                TokenType::Delimiter(Delimiter::Comma) => {
                    continue;
                }
                _ => {
                    self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected ) or ,"), Some(tuple_token.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                }
            }
        }
    }

    pub fn parse_type(&mut self) -> Option<TypeSyntax> {
        let token = self.tokens.next();
        match token.token_type() {
            TokenType::Atom(Atom::Type(t)) => return Some(TypeSyntax::Atom(t.to_owned())),
            TokenType::Identifier(s) => return Some(TypeSyntax::Defined { structure: s.to_owned() }),
            _ => {
                self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected type"), Some(token.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                return None;
            }
        }
    }

    pub fn parse_code_block(&mut self) -> Option<Vec<CodeSyntax>> {
        let mut code_syntax = vec![];
        loop {
            match self.tokens.peek()?.token_type() {
                TokenType::Delimiter(Delimiter::Brace(Brace::Curly(BraceState::Closed))) => {
                    self.tokens.next(); //Consume
                    return Some(code_syntax);
                }
                TokenType::Delimiter(Delimiter::Semicolon) => { 
                    self.tokens.next(); //Consume
                    self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Warning, format!("Unneeded semicolon"), Some(self.tokens.peek()?.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                }
                _ => {
                    if let Some(s) = self.parse_statement() {
                        code_syntax.push(s);
                    }
                }
            }
        }
    }

    pub fn parse_statement(&mut self) -> Option<CodeSyntax> {
        let statement = self.tokens.next();
        match statement.token_type() {
            TokenType::Keyword(Keyword::Let) => {
                let name = self.tokens.next();
                assert!(name.token_type().is_identifier());
                let eq = self.tokens.next();
                assert_eq!(&TokenType::Delimiter(Delimiter::Equals), eq.token_type());
                let value = self.parse_node_value()?;
                return Some(CodeSyntax::Let { variable: name.into_token_type().into_identifier().unwrap(), value });
            }
            TokenType::Keyword(Keyword::Force) => {
                let value = self.parse_node_value()?;
                let arrow = self.tokens.next();
                assert_eq!(arrow.into_token_type(), TokenType::ThickArrowRight);
                let type_syntax = self.parse_type()?;
                return Some(CodeSyntax::Force { value, type_syntax }); 
            }
            _ => {
                self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Unexpected token"), Some(statement.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                return None;
            }
        }
    }

    pub fn parse_node_value(&mut self) -> Option<NodeValueSyntax> {
        let node_value_token = self.tokens.next();
    
        match node_value_token.token_type() {
            TokenType::Atom(Atom::Type(t)) => {
                return Some(NodeValueSyntax::Literal(t.to_owned()))
            }
            TokenType::Identifier(structure) if self.tokens.peek().clone().and_then(|f| f.token_type().as_delimiter()).and_then(|f| if f.is_double_colon() {Some(())} else {None}).is_some() => {
                self.tokens.next();
                let sub = self.tokens.next();
                let sub = match sub.token_type() {
                    TokenType::Identifier(n) => n.to_owned(),
                    _ => {
                        panic!()
                    }
                };
                let application = self.parse_node_value().unwrap();
                let syntax = SubCallSyntax {
                    application: Some(application),
                    location: SubLocation::Structure {
                        collection: structure.to_owned(),
                        sub
                    }
                };
                return Some(NodeValueSyntax::Sub(syntax.into()))

            }
            TokenType::Atom(Atom::Sub(sub)) => {
                let application = self.parse_node_value().unwrap();
                let syntax = SubCallSyntax {
                    application: Some(application),
                    location: SubLocation::Atom(sub.to_owned())
                };
                return Some(NodeValueSyntax::Sub(syntax.into()))

            }
            TokenType::Identifier(name) => {
                return Some(NodeValueSyntax::Variable(name.to_owned()));
            }
            TokenType::Delimiter(Delimiter::Brace(Brace::Round(BraceState::Open))) => {
                let mut enumeration = vec![];

                loop {
                    match self.parse_node_value() {
                        Some(s) => enumeration.push(s),
                        None => {            
                            self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected )"), None, DiagnosticPipelineLocation::Parsing));
                        }

                    }
                    let tuple_token = self.tokens.next();
                    match tuple_token.token_type() {
                        TokenType::Delimiter(Delimiter::Brace(Brace::Round(BraceState::Closed))) => {
                            return Some(NodeValueSyntax::Tuple(enumeration));
                        }
                        TokenType::Delimiter(Delimiter::Comma) => {
                            continue;
                        }
                        _ => {
                            self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected ) or ,"), Some(tuple_token.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));

                        }
                    }
                }
            }
            _ => panic!(),

        }


        todo!()
        
    }
}

