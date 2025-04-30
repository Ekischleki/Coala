use std::collections::HashMap;

use crate::compiler::{block_parser::{Block, TokenBlock, TokenBlockType}, code_location::CodeLocation, compilation::Compilation, diagnostic::{Diagnostic, DiagnosticPipelineLocation, DiagnosticType}, syntax::{CodeSyntax, CollectionSyntax, CompositeTypeSyntax, ExpressionSyntax, FieldAssignSyntax, SubCallSyntax, SubLocation, SubstructureSyntax, TypeSyntax, TypedIdentifierSyntax}, token::{self, Atom, Brace, BraceState, Delimiter, Keyword, Token, TokenType}, type_stream::TypeStream};

pub struct Parser<'a> {
    //tokens: TypeStream<TokenBlock>,
    compilation: &'a mut Compilation,
    pub composite_types: Vec<CompositeTypeSyntax>,
    pub problems: Vec<SubstructureSyntax>,
    pub collections: Vec<CollectionSyntax>,
    pub solutions: HashMap<String, SubCallSyntax>,
    pub supers: HashMap<String, usize>
}


impl<'a> Parser<'a> {
    pub fn new(compilation: &'a mut Compilation) -> Self {
        Self {
            compilation,
            composite_types: vec![],
            problems: vec![],
            collections: vec![],
            supers: HashMap::default(),
            solutions: HashMap::default()
        }
    }

    pub fn parse_file(&mut self, token_stream: &mut TypeStream<TokenBlock>) {
        
        while !token_stream.is_empty() {
            let current_token = token_stream.next();
            match current_token.token_type() {
                TokenBlockType::Token(TokenType::Keyword(Keyword::Super)) => {
                     self.parse_super(token_stream);
                }
                TokenBlockType::Token(TokenType::Keyword(Keyword::Solution)) => {
                    if let None = token_stream.error_if_empty(self.compilation, "code block") {
                        continue;
                    }

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
                    if let None = token_stream.error_if_empty(self.compilation, "code block") {
                        continue;
                    }

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
    fn parse_composite(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<()> {
        token_stream.error_if_empty(self.compilation, "identifier")?;

        let identifier = token_stream.next().into_identifier_or_error(self.compilation)?;

        token_stream.error_if_empty(self.compilation, "code block")?;

        let composite_body = token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly)?;

        let fields = self.parse_typed_identifiers(composite_body)?;

        self.composite_types.push(CompositeTypeSyntax { name: identifier, fields });


        Some(())
    }
    fn parse_super(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<()> {
        let identifier = token_stream.next();
        let identifier_location = identifier.code_location().to_owned();
        let identifier = identifier.into_identifier_or_error(self.compilation)?;
        token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Equals);
        let int = token_stream.next().into_integer_or_error(self.compilation)?;

        if self.supers.contains_key(&identifier) {
            self.compilation.add_error("The name of this super is already in use", Some(identifier_location));
            return None;
        }
        self.supers.insert(identifier, int);

        Some(())
    }
    fn parse_problem(&mut self, block: Block) {
        let mut problems = vec![];
        self.parse_sub_collection(block,&mut problems);
        self.problems.append(&mut problems);
    }

    fn parse_collection(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<CollectionSyntax> {
//        let collection_token = self.tokens.next();
        token_stream.error_if_empty(self.compilation, "identifier")?;

        let name = token_stream.next().into_identifier_or_error(self.compilation)?;

        token_stream.error_if_empty(self.compilation, "code block")?;

        let collection_block = token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly)?;

        let mut subs = vec![];
        self.parse_sub_collection(collection_block, &mut subs);

        return Some(CollectionSyntax {
            subs,
            name
        })

    }

    fn parse_solution(&mut self, block: Block) -> Option<()> {
        let mut token_stream = TypeStream::from_iter(block.body.into_iter(), block.close_token.map(|f| f.code_location().to_owned()));
        while !token_stream.is_empty() {

            let function_name = match token_stream.next().into_identifier_or_error(self.compilation) {
                Some(s) => s,
                None => continue
            };
            token_stream.error_if_empty(self.compilation, "application")?;

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
        Some(())
    }

    //Parses the subs in a collection
    fn parse_sub_collection(&mut self, block: Block, subs: &mut Vec<SubstructureSyntax>) {
        let mut block_tokens = TypeStream::from_iter(block.body.into_iter(), block.close_token.map(|f| f.code_location().to_owned()));
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

        token_stream.error_if_empty(self.compilation, "identifier")?;

        let name = token_stream.next().into_identifier_or_error(self.compilation)?;

        token_stream.error_if_empty(self.compilation, "args")?;

        let args = token_stream.next().into_block_type_or_error(self.compilation, Brace::Round)?;

        let args = self.parse_typed_identifiers(args)?;

        token_stream.error_if_empty(self.compilation, "code block")?;

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
        let mut token_stream = TypeStream::from_iter(block.body.into_iter(), block.close_token.map(|f| f.code_location().to_owned()));
        while !token_stream.is_empty() {
            let type_syntax = self.parse_type(&mut token_stream)?;

            token_stream.error_if_empty(self.compilation, "colon")?;

            //Colon
            token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Colon)?;

            token_stream.error_if_empty(self.compilation, "identifier")?;


            let name = token_stream.next().into_identifier_or_error(self.compilation)?;

            
            enumeration.push(TypedIdentifierSyntax {type_syntax, name});

            if token_stream.is_empty() {
                break;
            }

            token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Comma);
        }
        return Some(enumeration);
    }

    pub fn parse_type(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<TypeSyntax> {
        token_stream.error_if_empty(self.compilation, "type")?;

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

        let mut token_stream = TypeStream::from_iter(block.body.into_iter(), block.close_token.map(|f| f.code_location().to_owned()));

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

        token_stream.error_if_empty(self.compilation, "statement")?;

        let statement = token_stream.next();
        match statement.token_type() {
            TokenBlockType::Token(TokenType::Keyword(Keyword::If)) => {

                token_stream.error_if_empty(self.compilation, "code block")?;
                let condition_block = token_stream.next().into_block_type_or_error(self.compilation, Brace::Round)?;
                let mut condition_stream = TypeStream::from_iter(condition_block.body.into_iter(), condition_block.close_token.map(|s| s.code_location().to_owned())); 
                
                let condition = self.parse_expression(&mut condition_stream)?;

                if !condition_stream.is_empty() {
                    self.compilation.add_error("Unexpected token", Some(condition_stream.next().code_location().to_owned()));
                }

                token_stream.error_if_empty(self.compilation, "code block")?;
                let condition_true = token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly)?;
                let condition_true = self.parse_code_block(condition_true).unwrap_or_default();

                if let Some(TokenBlockType::Token(TokenType::Keyword(Keyword::Else))) = token_stream.peek().map(|s| s.token_type()) {
                    token_stream.next();
                    token_stream.error_if_empty(self.compilation, "code block")?;
                    let condition_false = token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly)?;
                    let condition_false = self.parse_code_block(condition_false).unwrap_or_default();                
                    Some(CodeSyntax::IfElse { condition, condition_true, condition_false })
                } else {
                    Some(CodeSyntax::If { condition, condition_true })
                }

            }
            TokenBlockType::Token(TokenType::Keyword(Keyword::Let)) => {
                token_stream.error_if_empty(self.compilation, "identifier")?;

                let variable = token_stream.next().into_identifier_or_error(self.compilation)?;

                token_stream.error_if_empty(self.compilation, "=")?;


                token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Equals);
                let value = self.parse_expression(token_stream)?;
                return Some(CodeSyntax::Let { variable, value });
            }
            TokenBlockType::Token(TokenType::Keyword(Keyword::Force)) => {
                let value = self.parse_expression(token_stream)?;
                token_stream.error_if_empty(self.compilation, "=>")?;

                //Arrow
                token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::ThickArrowRight);

                let type_syntax = self.parse_type(token_stream)?;
                return Some(CodeSyntax::Force { value, type_syntax }); 
            }
            TokenBlockType::Token(TokenType::Keyword(Keyword::Output)) => {
                let expression = self.parse_expression(token_stream)?;
                return Some(CodeSyntax::Output { expression })
            }
            TokenBlockType::Token(TokenType::Identifier(structure)) if token_stream.peek().is_some_and(|s| s.token_type().is_double_colon()) => {
                token_stream.next();
                token_stream.error_if_empty(self.compilation, "identifier")?;

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

    pub fn parse_expression(&mut self, token_stream: &mut TypeStream<TokenBlock>) -> Option<ExpressionSyntax> {

        token_stream.error_if_empty(self.compilation, "expression")?;


        let node_value_token = token_stream.next();
    
        match node_value_token.token_type() {
            TokenBlockType::Token(TokenType::String(_)) => {
                return Some(ExpressionSyntax::String(node_value_token.into_string_or_error(self.compilation)?));
            }

            TokenBlockType::Token(TokenType::Atom(Atom::Type(t))) => {
                return Some(ExpressionSyntax::Literal(t.to_owned()))
            }

            TokenBlockType::Token(TokenType::Identifier(structure)) if token_stream.peek().is_some_and(|s| s.token_type().is_double_colon()) => {
                
                token_stream.next();
                token_stream.error_if_empty(self.compilation, "identifier")?;

                let sub = token_stream.next().into_identifier_or_error(self.compilation)?;

                let application = self.parse_expression(token_stream)?;
                let syntax = SubCallSyntax {
                    application: Some(application),
                    location: SubLocation::Structure {
                        collection: structure.to_owned(),
                        sub
                    }
                };
                return Some(ExpressionSyntax::Sub(syntax.into()))

            }

            TokenBlockType::Token(TokenType::Atom(Atom::Sub(sub))) => {
                let application = self.parse_expression(token_stream)?;
                let syntax = SubCallSyntax {
                    application: Some(application),
                    location: SubLocation::Atom(sub.to_owned())
                };
                return Some(ExpressionSyntax::Sub(syntax.into()))

            }
            TokenBlockType::Token(TokenType::Identifier(type_name)) if token_stream.peek().is_some_and(|s| s.token_type().is_curly_block()) => {
                token_stream.error_if_empty(self.compilation, "code block")?;

                let assign_block = token_stream.next().into_block_type_or_error(self.compilation, Brace::Curly)?;
                let field_assign = self.parse_field_assign(assign_block)?;
                return Some(ExpressionSyntax::CompositeConstructor { type_name: type_name.to_owned(), field_assign })

            }
            TokenBlockType::Token(TokenType::Identifier(base)) if token_stream.peek().is_some_and(|s| s.token_type().is_period()) => {
                let mut chain = ExpressionSyntax::Variable(base.to_owned());

                while let Some(TokenBlockType::Token(TokenType::Delimiter(Delimiter::Period))) = token_stream.peek().map(|s| s.token_type()) {
                    token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Period)?;


                    token_stream.error_if_empty(self.compilation, "identifier")?;

                    let access_token = token_stream.next();

                    chain = match access_token.token_type() {
                        TokenBlockType::Token(TokenType::Integer(idx)) => {
                            ExpressionSyntax::AccessIdx { base: chain.into(), idx: *idx }
                        },
                        TokenBlockType::Token(TokenType::Identifier(_)) => {
                            let identifier: String = access_token.into_identifier_or_error(self.compilation)?;
                            ExpressionSyntax::Access { base: chain.into(), field: identifier }
                        }
                        _ => {
                            self.compilation.add_error("Expected integer or identifier", Some(access_token.code_location().to_owned()));
                            return None;
                        }
                    };

                }
                return Some(chain);
            }
            TokenBlockType::Token(TokenType::Identifier(name)) => {

                return Some(ExpressionSyntax::Variable(name.to_owned()));
            }
            TokenBlockType::Block(block) if block.is_round() => {
                let block = match node_value_token.into_block_type_or_error(self.compilation, Brace::Round) {
                    Some(s) => s,
                    None => {
                        println!("You shouldn't be here!");
                        return None;
                    }
                };

                let mut token_stream = TypeStream::from_iter(block.body.into_iter(), block.close_token.map(|f| f.code_location().to_owned()));

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
                return Some(ExpressionSyntax::Tuple(enumeration));
            }
            _ => {
                self.compilation.add_error("Expected expression", Some(node_value_token.code_location().to_owned()));
                return None;
            },

        }        
    }

    pub fn parse_field_assign(&mut self, block: Block) -> Option<Vec<FieldAssignSyntax>> {
        let mut res = vec![];
        let mut token_stream = TypeStream::from_iter(block.body.into_iter(), block.close_token.map(|f| f.code_location().to_owned()));

        while !token_stream.is_empty() {
            let field_name = token_stream.next().into_identifier_or_error(self.compilation)?;

            token_stream.error_if_empty(self.compilation, ":")?;

            token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Colon);

            let expression = self.parse_expression(&mut token_stream)?;

            res.push(FieldAssignSyntax { left: field_name, right: expression });

            if token_stream.is_empty() {
                break;
            }

            token_stream.next().assert_is_delimiter_or_error(self.compilation, Delimiter::Comma);



        }

        Some(res)
    }
}

