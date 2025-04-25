use std::collections::HashMap;

use crate::{block_parser::{Block, TokenBlock, TokenBlockType}, code_location::CodeLocation, compilation::Compilation, diagnostic::{Diagnostic, DiagnosticPipelineLocation, DiagnosticType}, syntax::{CodeSyntax, CollectionSyntax, CompositeTypeSyntax, NodeValueSyntax, SubCallSyntax, SubLocation, SubstructureSyntax, TypeSyntax, TypedIdentifierSyntax}, token::{Atom, Brace, BraceState, Delimiter, Keyword, Token, TokenType}, type_stream::TypeStream};

pub struct Parser<'a> {
    //tokens: TypeStream<TokenBlock>,
    compilation: &'a mut Compilation,
    pub composite_types: Vec<CompositeTypeSyntax>,
    pub problems: Vec<SubstructureSyntax>,
    pub collections: Vec<CollectionSyntax>,
    pub solutions: HashMap<String, SubCallSyntax>
}


impl<'a> Parser<'a> {
    pub fn new(compilation: &'a mut Compilation) -> Self {
        Self {
            compilation,
            composite_types: vec![],
            problems: vec![],
            collections: vec![],
            solutions: HashMap::default()
        }
    }

    pub fn parse_file(&mut self, token_stream: &mut TypeStream<TokenBlock>) {
        
        while !token_stream.is_empty() {
            let current_token = token_stream.next();
            match current_token.token_type() {
                TokenBlockType::Token(TokenType::Keyword(Keyword::Solution)) => {
                    let solution_block = match token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly){
                        Some(s) => s,
                        None => continue
                    };
                    self.parse_solution(solution_block);
                }
                TokenBlockType::Token(TokenType::Keyword(Keyword::Collection)) => {

                    if let Some(c) = self.parse_collection(token_stream) {
                        self.collections.push(c);
                    }
                }
                TokenBlockType::Token(TokenType::Keyword(Keyword::Problem)) => {
                    let problem_block = match token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly){
                        Some(s) => s,
                        None => continue
                    };
                    self.parse_problem(problem_block);
                }
                TokenBlockType::Token(TokenType::Keyword(Keyword::Composite)) => {
                    self.parse_composite(token_stream);
                }
                TokenBlockType::Token(TokenType::EOF) => return,
                _ => {            
                    self.compilation.add_error("Unexpected token at file level", Some(current_token.code_location().to_owned()));
                }
            }
        }
    }
    fn parse_composite(&mut self, token_stream: &mut TypeStream<TokenBlock>) {
        let identifier = match token_stream.next().into_identifier_or_error(self.compilation) {
            Some(s) => s,
            None => return
        };

        let composite_body = match token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly) {
            Some(s) => s,
            None => return
        };

        let fields = match self.parse_typed_identifiers(composite_body) {
            Some(s) => s,
            None => return
        };

        self.composite_types.push(CompositeTypeSyntax { name: identifier, fields });



    }

    fn parse_problem(&mut self, block: Block) {
        let mut problems = vec![];
        self.parse_sub_collection(block,&mut problems);
        self.problems.append(&mut problems);
    }

    fn parse_collection(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<CollectionSyntax> {
//        let collection_token = self.tokens.next();
        let name = token_stream.next().into_identifier_or_error(self.compilation)?;

        let collection_block = token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly)?;

        let mut subs = vec![];
        self.parse_sub_collection(collection_block, &mut subs);

        return Some(CollectionSyntax {
            subs,
            name
        })

    }

    fn parse_solution(&mut self, block: Block) {
        let mut token_stream = TypeStream::from_iter(block.body.into_iter());
        while !token_stream.is_empty() {

            let function_name = match token_stream.next().into_identifier_or_error(self.compilation) {
                Some(s) => s,
                None => continue
            };
            if token_stream.is_empty() {
                self.compilation.add_error("Expected application", block.close_token.map(|t| t.code_location().to_owned()));
                break;
            }
            let application = match self.parse_expression(&mut token_stream) {
                Some(n) => n,
                None => continue
            };
            
            self.solutions.insert(function_name.to_owned(), SubCallSyntax { location: SubLocation::Structure { collection: String::new(), sub: function_name }, application: Some(application) });
            
            if token_stream.is_empty() {
                break;
            }
            
            token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Comma);
        } 
    }

    //Parses the subs in a collection
    fn parse_sub_collection(&mut self, block: Block, subs: &mut Vec<SubstructureSyntax>) {
        let mut block_tokens = TypeStream::from_iter(block.body.into_iter());
        while !block_tokens.is_empty() {
            let cur_token = block_tokens.next();
            match cur_token.token_type() {
                TokenBlockType::Token(TokenType::Keyword(Keyword::SubStructure)) => {
                    if let Some(s) = self.parse_sub(&mut block_tokens) {
                        subs.push(s);
                    }
                }
                _ => {
                    self.compilation.add_error("Unexpected token within collection", Some(cur_token.code_location().to_owned()));
                }
            }
        
        }
    }
    
    pub fn parse_sub(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<SubstructureSyntax> {


        let name = token_stream.next().into_identifier_or_error(self.compilation)?;

        let args = token_stream.next().into_block_type_or_error(self.compilation, Brace::Round)?;

        let args = self.parse_typed_identifiers(args)?;

        let code = token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly)?;

        let code = self.parse_code_block(code)?;

        let mut result = None;
        if let Some(TokenBlockType::Token(TokenType::Delimiter(Delimiter::Equals))) = token_stream.peek().map(|t| t.token_type()) {
            token_stream.next();
            result = self.parse_expression(token_stream);
        }
        Some(SubstructureSyntax { name, args, code, result })
    }

    pub fn parse_typed_identifiers(&mut self, block: Block) -> Option<Vec<TypedIdentifierSyntax>> {
        let mut enumeration = vec![];
        let mut tokens = TypeStream::from_iter(block.body.into_iter());
        while !tokens.is_empty() {
            let type_syntax = self.parse_type(&mut tokens)?;

            if tokens.is_empty() {
                self.compilation.add_error("Expected colon", block.close_token.map(|t| t.code_location().to_owned()));
                break;
            }
            //Colon
            tokens.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Colon)?;

            if tokens.is_empty() {
                self.compilation.add_error("Expected identifier", block.close_token.map(|t| t.code_location().to_owned()));
                break;
            }

            let name = tokens.next().into_identifier_or_error(self.compilation)?;

            
            enumeration.push(TypedIdentifierSyntax {type_syntax, name});

            if tokens.is_empty() {
                break;
            }

            tokens.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Comma);
        }
        return Some(enumeration);
    }

    pub fn parse_type(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<TypeSyntax> {
        let token = token_stream.next();
        match token.token_type() {
            TokenBlockType::Token(TokenType::Atom(Atom::Type(t))) => return Some(TypeSyntax::Atom(t.to_owned())),
            TokenBlockType::Token(TokenType::Identifier(_)) => return Some(TypeSyntax::Composite { name: token.into_identifier_or_error(self.compilation)? }),
            _ => {
                self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Expected type"), Some(token.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                return None;
            }
        }
    }

    pub fn parse_code_block(&mut self, block: Block) -> Option<Vec<CodeSyntax>> {
        let mut code_syntax = vec![];

        let mut token_stream = TypeStream::from_iter(block.body.into_iter());

        loop {
            match token_stream.peek().map(|f| f.token_type()) {
                None => {
                    return Some(code_syntax);
                }
                Some(TokenBlockType::Token(TokenType::Delimiter(Delimiter::Semicolon))) => { 
                    token_stream.next(); //Consume
                    self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Warning, format!("Unneeded semicolon"), Some(token_stream.peek()?.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                }
                _ => {
                    if let Some(s) = self.parse_statement(&mut token_stream) {
                        code_syntax.push(s);
                    }
                }
            }
        }
    }

    pub fn parse_statement(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<CodeSyntax> {
        let statement = token_stream.next();
        match statement.token_type() {
            TokenBlockType::Token(TokenType::Keyword(Keyword::Let)) => {
                let variable = token_stream.next().into_identifier_or_error(self.compilation)?;
                token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Equals);
                let value = self.parse_expression(token_stream)?;
                return Some(CodeSyntax::Let { variable, value });
            }
            TokenBlockType::Token(TokenType::Keyword(Keyword::Force)) => {
                let value = self.parse_expression(token_stream)?;
                //Arrow
                token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::ThickArrowRight);

                let type_syntax = self.parse_type(token_stream)?;
                return Some(CodeSyntax::Force { value, type_syntax }); 
            }
            TokenBlockType::Token(TokenType::Identifier(structure)) if token_stream.peek().is_some_and(|s| s.token_type().is_double_colon()) => {
                token_stream.next();
                let sub = token_stream.next().into_identifier_or_error(self.compilation)?;

                let application = self.parse_expression(token_stream)?;
                let syntax = SubCallSyntax {
                    application: Some(application),
                    location: SubLocation::Structure {
                        collection: structure.to_owned(),
                        sub
                    }
                };
                return Some(CodeSyntax::Sub(syntax.into()))

            }
            _ => {
                self.compilation.add_diagnostic(Diagnostic::new(DiagnosticType::Error, format!("Unexpected token"), Some(statement.code_location().to_owned()), DiagnosticPipelineLocation::Parsing));
                return None;
            }
        }
    }

    pub fn parse_expression(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<NodeValueSyntax> {
        let node_value_token = token_stream.next();
    
        match node_value_token.token_type() {
            TokenBlockType::Token(TokenType::Atom(Atom::Type(t))) => {
                return Some(NodeValueSyntax::Literal(t.to_owned()))
            }

            TokenBlockType::Token(TokenType::Identifier(structure)) if token_stream.peek().is_some_and(|s| s.token_type().is_double_colon()) => {
                token_stream.next();
                let sub = token_stream.next().into_identifier_or_error(self.compilation)?;

                let application = self.parse_expression(token_stream)?;
                let syntax = SubCallSyntax {
                    application: Some(application),
                    location: SubLocation::Structure {
                        collection: structure.to_owned(),
                        sub
                    }
                };
                return Some(NodeValueSyntax::Sub(syntax.into()))

            }

            TokenBlockType::Token(TokenType::Atom(Atom::Sub(sub))) => {
                let application = self.parse_expression(token_stream)?;
                let syntax = SubCallSyntax {
                    application: Some(application),
                    location: SubLocation::Atom(sub.to_owned())
                };
                return Some(NodeValueSyntax::Sub(syntax.into()))

            }
            TokenBlockType::Token(TokenType::Identifier(base)) if token_stream.peek().is_some_and(|s| s.token_type().is_period()) => {
                todo!();
                return Some(NodeValueSyntax::Variable(base.to_owned()));
            }
            TokenBlockType::Token(TokenType::Identifier(name)) => {

                return Some(NodeValueSyntax::Variable(name.to_owned()));
            }
            TokenBlockType::Block(block) if block.is_round() => {
                let block = match node_value_token.into_block_type_or_error(self.compilation, Brace::Round) {
                    Some(s) => s,
                    None => {
                        println!("You shouldn't be here!");
                        return None;
                    }
                };

                let mut token_stream = TypeStream::from_iter(block.body.into_iter());

                let mut enumeration = vec![];

                while !token_stream.is_empty() {

                    match self.parse_expression(&mut token_stream) {
                        Some(s) => enumeration.push(s),
                        None => {}
                    }
                    if token_stream.is_empty() {
                        break;
                    }
                    let tuple_token = token_stream.next();
                    match tuple_token.token_type() {
                        TokenBlockType::Token(TokenType::Delimiter(Delimiter::Comma)) => {
                            continue;
                        }
                        _ => {
                            self.compilation.add_error("Expected ) or ,", Some(tuple_token.code_location().to_owned()));
                        }
                    }
                }
                return Some(NodeValueSyntax::Tuple(enumeration));
            }
            _ => {
                self.compilation.add_error("Expected expression", Some(node_value_token.code_location().to_owned()));
                return None;
            },

        }        
    }
}

