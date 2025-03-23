use crate::{compilation::Compilation, diagnostic::{Diagnostic, DiagnosticPipelineLocation, DiagnosticType}, graph_structure_type::{NodeValueSyntax, StructureSymbol, SubCallSyntax}, symbol_table::{ContextSymbolTable, GlobalSymbolTable}, token::{Brace, BraceState, Delimiter, Keyword, Token, TokenType}, type_stream::TypeStream};

pub struct Parser<'a> {
    type_stream: TypeStream<Token>,
    compilation: &'a mut Compilation,
    symbol_table: GlobalSymbolTable
}


impl Parser<'_> {
    fn parse_file(&mut self) {
        while let Some(current_token) = self.type_stream.peek() {
            match current_token.token_type() {
                TokenType::Keyword(Keyword::Structure) => {
                }
                _ => {panic!()}
            }
        }
    }

    fn parse_structure(&mut self) -> Option<StructureSymbol> {
        let struct_token = self.type_stream.next();
        let structure_identifier = self.type_stream.next();
        
        if let TokenType::Identifier(name) = structure_identifier.into_token_type() {
            let name = name.to_owned();
        } else {
            self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected Identifier after structure keyword"), Some(struct_token.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
            return None;
        }

        let open_curly_delim = self.type_stream.next();
        assert!(open_curly_delim.token_type().as_delimiter().unwrap().as_brace().unwrap().as_curly().unwrap().is_open());
        loop {
                        
        }


        todo!()
    }


    fn parse_code() {
        todo!()
    }

    fn parse_node_value(&mut self, context: &ContextSymbolTable) -> Option<NodeValueSyntax> {
        let node_value_token = self.type_stream.next();
    
        match node_value_token.token_type() {
            TokenType::Identifier(structure) if self.type_stream.peek().clone().unwrap().token_type().as_delimiter().unwrap().is_double_colon() => {
                self.type_stream.next();
                let sub = self.type_stream.next();
                let sub = match sub.token_type() {
                    TokenType::Identifier(n) => n.to_owned(),
                    _ => {
                        panic!()
                    }
                };
                let application = self.parse_node_value(context).unwrap();
                let syntax = SubCallSyntax {
                    application: Some(application),
                    structure: structure.to_owned(),
                    sub
                };
                return Some(NodeValueSyntax::Sub(syntax.into()))

            }
            TokenType::Identifier(name) => {
                return Some(NodeValueSyntax::Variable(self.type_stream.next().into_token_type().into_identifier().unwrap()));
            }
            TokenType::Delimiter(Delimiter::Brace(Brace::Round(BraceState::Open))) => {
                let mut enumeration = vec![];
                loop {
                    match self.parse_node_value(context) {
                        Some(s) => enumeration.push(s),
                        None => {            
                            self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected )"), None, DiagnosticPipelineLocation::Parsing));
                        }

                    }
                    let tuple_token = self.type_stream.next();
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

