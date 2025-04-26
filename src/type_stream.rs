use std::{fmt::Debug, vec::IntoIter};

use crate::{block_parser::TokenBlock, code_location::CodeLocation, compilation::{self, Compilation}, token::Token};

#[derive(Debug)]
pub struct TypeStream<T: Debug> {
    tokens: IntoIter<T>,
    next: Option<T>,
    end: Option<CodeLocation>
}


impl<T: Debug> Iterator for TypeStream<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_some() {
            Some(self.next())
        } else {
            None
        }
    }
}



impl<T: Debug> TypeStream<T> {
    pub fn error_if_empty(&self, compilation: &mut Compilation, expected: &str) -> Option<()> {
        if self.is_empty() {
            compilation.add_error(&format!("Expected {expected}"), self.end.clone());
            None
        } else {
            Some(())
        }
    }
    pub fn is_empty(&self) -> bool {
        self.next.is_none()
    }
    pub fn new(tokens: Vec<T>) -> Self{

        Self::from_iter(tokens.into_iter(), None)
    }

    pub fn to_vec(self) -> Vec<T> {
        self.into()
    }
    pub fn from_iter(iter: IntoIter<T>, end: Option<CodeLocation>) -> Self {
        let mut res = Self {
            tokens: iter,
            next: None,
            end
        };
        res.next = res.tokens.next();
        return res;
    }
    
    pub fn next(&mut self) -> T {
        
        match self.next.take() {
            Some(t) => {
                println!("Consumed {:#?}", t);
                self.next = self.tokens.next();
                return t;
            }
            None => {
                panic!("Oh no, the typed stream is empty qwq\nSeems like SOMEBODY hasn't regarded the EOF token...")
            }
        }
    }
    ///Performs an operation with the upcoming item as input. This can be used as a replacement for `.peek()`
    pub fn extract<U>(&self, extractor: fn (&T) -> U) -> U {
        extractor(&self.next.as_ref().expect("Typed stream was empty."))
    }

    pub fn peek(&self) -> Option<&T> {
        self.next.as_ref()
    }

    pub fn skip(&mut self) {
        _ = self.next();
    }
}


impl<T: Debug> Into<Vec<T>> for TypeStream<T> {
    fn into(self) -> Vec<T> {
        if let Some(next) = self.next {
            std::iter::once(next).chain(self.tokens).collect()
        } else {
            self.tokens.collect()
        }
    }
}