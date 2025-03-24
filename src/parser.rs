use crate::{compilation::Compilation, diagnostic::{Diagnostic, DiagnosticPipelineLocation, DiagnosticType}, graph_structure_type::{CodeSyntax, NodeValueSyntax, StructureSymbol, SubCallSyntax}, symbol_table::{self, ContextSymbolTable, GlobalSymbolTable}, token::{Brace, BraceState, Delimiter, Keyword, Token, TokenType}, type_stream::TypeStream};

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

    fn parse_structure(&mut self) -> Option<StructureSymbol> {
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


    fn parse_code(&mut self) -> Option<CodeSyntax> {
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

            _ => {
                self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Unexpected token"), Some(statement.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                return None;
            }
        }
    }

    pub fn parse_node_value(&mut self) -> Option<NodeValueSyntax> {
        let node_value_token = self.tokens.next();
    
        match node_value_token.token_type() {
            TokenType::Identifier(structure) if self.tokens.peek().clone().unwrap().token_type().as_delimiter().unwrap().is_double_colon() => {
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
                    structure: structure.to_owned(),
                    sub
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

