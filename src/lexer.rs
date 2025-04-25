
use std::path::PathBuf;

use phf::{phf_map, phf_set};


use crate::token::{Atom, AtomSub, AtomType, Brace, BraceState, Delimiter, Keyword};

use super::{code_location::CodeLocation, compilation::Compilation, diagnostic::{Diagnostic, DiagnosticPipelineLocation, DiagnosticType}, file_reader::{FileReader, FileReaderError}, token::{Token, TokenType}, type_stream::TypeStream};


static ESCAPE_MAPPING: phf::Map<&'static str, &'static str> = phf_map! {
    "n" => "\n",
    "t" => "\t",
    "\\" => "\\",
};

static IGNORE_CHARS: phf::Set<char> = phf_set! {
    ' ',
    '\t',
    '\n',
    '\r'
};

static KEYWORD_MAPPING: phf::Map<&'static str, &'static TokenType> = phf_map! {
    "problem" => &TokenType::Keyword(Keyword::Problem),
    "solution" => &TokenType::Keyword(Keyword::Solution),

    "let" => &TokenType::Keyword(Keyword::Let),
    "force" => &TokenType::Keyword(Keyword::Force),

    "sub" => &TokenType::Keyword(Keyword::SubStructure),
    "collection" => &TokenType::Keyword(Keyword::Collection),
    "composite" => &TokenType::Keyword(Keyword::Composite),


    "true" => &TokenType::Atom(Atom::Type(AtomType::True)),
    "false" => &TokenType::Atom(Atom::Type(AtomType::False)),

    "not" => &TokenType::Atom(Atom::Sub(AtomSub::Not)),
    "or" => &TokenType::Atom(Atom::Sub(AtomSub::Or)),

};

static DELIM_MAPPING: phf::Map<&'static str, &'static TokenType> = phf_map! {
    "->" => &TokenType::ThinArrowRight,
    "=>" => &TokenType::ThickArrowRight,

    "(" => &TokenType::Delimiter(Delimiter::Brace(Brace::Round(BraceState::Open))),
    ")" => &TokenType::Delimiter(Delimiter::Brace(Brace::Round(BraceState::Closed))),

    "{" => &TokenType::Delimiter(Delimiter::Brace(Brace::Curly(BraceState::Open))),
    "}" => &TokenType::Delimiter(Delimiter::Brace(Brace::Curly(BraceState::Closed))),

    "[" => &TokenType::Delimiter(Delimiter::Brace(Brace::Square(BraceState::Open))),
    "]" => &TokenType::Delimiter(Delimiter::Brace(Brace::Square(BraceState::Closed))),

    ":" => &TokenType::Delimiter(Delimiter::Colon),
    "::" => &TokenType::Delimiter(Delimiter::DoubleColon),

    ";" => &TokenType::Delimiter(Delimiter::Semicolon),
    "," => &TokenType::Delimiter(Delimiter::Comma),
    "." => &TokenType::Delimiter(Delimiter::Period),

    "=" => &TokenType::Delimiter(Delimiter::Equals),

};


    
/// If the file reader error was a EOF error, adds a EOF token to the token stream, and returns Some(token_stream)
/// If the file reader error was unusual, it adds a diagnostic and returns None, as the tokens don't matter if incomplete.
fn on_file_reader_error(e: FileReaderError, compilation: &mut Compilation, mut token_stream: Vec<Token>, current_file: &PathBuf) -> Option<TypeStream<Token>> {
    match e {
        FileReaderError::ReachedEOF => {
            token_stream.push(Token::eof(current_file));

            Some(TypeStream::new(token_stream))
        }
        
        FileReaderError::DiagnosticError(d) => {
            compilation.add_diagnostic(d);
            None
        }
    }
}

pub fn tokenize<T: FileReader>(file_reader: &mut T, current_file: &PathBuf, compilation: &mut Compilation) -> Option<TypeStream<Token>> {
    let mut token_stream = vec![];
    loop {
        let current_char;

        match file_reader.peek_char() {
            Ok(c) => {
                current_char = c;
            }
            Err(e) => {
                return on_file_reader_error(e, compilation, token_stream, current_file);
            }
        }
        
        
        match current_char {
            /*
            '"' => {
                match read_string(file_reader, current_file) {
                    Ok(s) => {
                        token_stream.push(s);
                    }
                    Err(e) => {
                        compilation.add_diagnostic(e);
                    }
                }
            }
             */
            '#' => {
                loop {
                    let comment_char;
                    match file_reader.read_char() {
                        Ok(c) => {
                            comment_char = c;
                        }
                        Err(e) => {
                            return on_file_reader_error(e, compilation, token_stream, current_file);
                        }
                    }
                    if comment_char == '\n' || comment_char == '\r' {
                        break;
                    }
                }
            }
            _ => {

                if IGNORE_CHARS.contains(&current_char) {
                    _ = file_reader.read_char(); //Jump to next char
                    continue;
                }
                match read_text(file_reader, current_file) {
                    Ok(t) => {
                        token_stream.push(t);
                    }
                    Err(e) => {
                        compilation.add_diagnostic(e);
                    }
                };

            }
        }
    };
}


fn read_text<T: FileReader>(file_reader: &mut T, current_file: &PathBuf) -> Result<Token, Diagnostic> {
    let first_char = file_reader.peek_char().expect("Unexpected file reader error");
    if first_char.is_ascii_digit() {
        read_number(file_reader, current_file)
    } else 
    if first_char.is_alphanumeric() {
        read_keyword(file_reader, current_file)
    } else {
        read_delim(file_reader, current_file)
    }
}

fn read_number(file_reader: &mut dyn FileReader, current_file: &PathBuf) -> Result<Token, Diagnostic> {
    let start_char = file_reader.get_position();
    let mut number = String::new();
    let mut number_char = file_reader.read_char().expect("Unintended file reading error.");
    while number_char.is_ascii_digit() {
        if IGNORE_CHARS.contains(&number_char) {
            break;
        }
        number.push(number_char);

        match file_reader.read_char() {
            Ok(c) => {
                number_char = c;
            }
            Err(_) => {
                break; //The current char implementation will take care of the error later :3. Def sounds like a ticking time-bomb, but not for me to worry about
            }
        }                    
    }
    file_reader.set_position(file_reader.get_position() - 1); //This should in theory not cause an underflow exception

    let parsed_int = number.parse();
    let location = CodeLocation::with_section(
        current_file.to_owned(),
        start_char,
        file_reader.get_position());

    let parsed_int = match parsed_int {
        Ok(u) => u,
        Err(e) => {
            return Err(Diagnostic::new(DiagnosticType::Error, format!("Couldn't parse integer: {e}"), Some(location), DiagnosticPipelineLocation::Lexing))
        }
    }; 

   let token_type = TokenType::Integer(parsed_int);

    return Ok(Token::new(
        token_type,
        location
    ));
}


fn read_delim<T: FileReader>(file_reader: &mut T, current_file: &PathBuf) -> Result<Token, Diagnostic> {
    let start_char = file_reader.get_position();
    let mut delim = String::with_capacity(2);
    loop {
        let delim_char;
        match file_reader.read_char() {
            Ok(c) => {
                delim_char = c;
            }
            Err(_) => {
                break; //The current char implementation will take care of the error later :3. Def sounds like a ticking time-bomb, but not for me to worry about
            }
        }  
        delim.push(delim_char);
        if try_get_delim(&delim).is_none() {
            delim.pop();
            break;
        }
    }
    if delim.len() != 0 {
        file_reader.set_position(file_reader.get_position() - 1);
    }
    let code_location = CodeLocation::with_section(current_file.to_owned(),
    start_char,
    file_reader.get_position()
    );
    let delim_type = try_get_delim(&delim);

    match delim_type {
        Some(token_type) => {
            return Ok(Token::new(
                token_type.clone(),
                code_location
                ));
        }
        None => {
            return Err(Diagnostic::new(
                DiagnosticType::Error,
                format!("Invalid delimitor"),
                Some(code_location),
                DiagnosticPipelineLocation::Lexing
            ));
        }
    }
}

fn read_keyword<T: FileReader>(file_reader: &mut T, current_file: &PathBuf) -> Result<Token, Diagnostic> {

    

    let start_char = file_reader.get_position();
    let mut keyword = String::new();
    let mut keyword_char = file_reader.read_char().expect("Unintended file reading error.");
    while keyword_char.is_alphanumeric() || keyword_char == '_' {
        if IGNORE_CHARS.contains(&keyword_char) {
            break;
        }
        keyword.push(keyword_char);

        match file_reader.read_char() {
            Ok(c) => {
                keyword_char = c;
            }
            Err(_) => {
                break; //The current char implementation will take care of the error later :3. Def sounds like a ticking time-bomb, but not for me to worry about
            }
        }                    
    }
    file_reader.set_position(file_reader.get_position() - 1); //This should in theory not cause an underflow exception

    let token_type = try_get_keyword(&keyword);
    let code_location = CodeLocation::with_section(current_file.to_owned(),
    start_char,
    file_reader.get_position()
    );
    match token_type {
        Some(token_type) => {
            return Ok(Token::new(
                token_type,
                code_location
                ));
        }
        None => {
            return Ok(Token::new(
            TokenType::Identifier(keyword),
            code_location
            ))
        }
    }
}

fn try_get_keyword(keyword: &String) -> Option<TokenType> {
    KEYWORD_MAPPING.get(&keyword).map(|f| (*f).to_owned())

}

fn try_get_delim(delim: &String) -> Option<TokenType> {
    DELIM_MAPPING.get(&delim).map(|f| (*f).to_owned())
}

/*
fn read_string(file_reader: &mut dyn FileReader, current_file: &PathBuf) -> Result<Token, Diagnostic> {

    let location_begin = file_reader.get_position();
        file_reader.set_position(file_reader.get_position() + 1); //We don't care about the initial '"'.

    let mut string = String::new();
    let mut read: char;
    loop {
        match file_reader.read_char() {
            Ok(c) => {read = c;}
            Err(e) => {
                match e {
                    FileReaderError::ReachedEOF => {
                        return Err(Diagnostic::new(
                            DiagnosticType::Error,
                            "Expected '\" to mark end of string'.".to_owned(),
                            Some(
                                CodeLocation::with_section(current_file.to_owned(), location_begin, file_reader.get_position())
                            ),
                            DiagnosticPipelineLocation::Lexing
                        ));
                    }
                    FileReaderError::DiagnosticError(d) => {
                        return Err(d);
                    }
                }
            }
        }
        if read == '"' {
            break;
        }
        if read == '\\' { //Escape, todo: currently no complex value insertations
            let escape_char ;
            match file_reader.read_char() {
                Ok(c) => {escape_char = c;}
                Err(e) => {
                    match e {
                        FileReaderError::ReachedEOF => {
                            return Err(Diagnostic::new(
                                DiagnosticType::Error,
                                "Expected followup character for escape, and end of string.".to_owned(),
                                Some(
                                    CodeLocation::with_section(current_file.to_owned(), location_begin, file_reader.get_position())
                                ),
                                DiagnosticPipelineLocation::Lexing
                            ));
                        }
                        FileReaderError::DiagnosticError(d) => {
                            return Err(d);
                        }
                    }
                }
            }
            let replace_char = ESCAPE_MAPPING[escape_char.to_string().as_str()];
            string.push_str(replace_char);
        }
        else {
            string.push(read);
        }
    }

    return Ok(
        Token::new(
            TokenType::ConstValue(ConstValue::String(string)),
            CodeLocation::with_section(
                current_file.to_owned(), 
                location_begin, 
                file_reader.get_position())));
}
 */