use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::types::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FileRead {
    All,
    Lines,
    Bytes,
}

pub struct IoFile {
    pub file: BufReader<File>,
    pos: usize,
    line: Option<Scalar>,
}

impl IoFile {
    pub fn new(file: File) -> Self {
        let file = BufReader::new(file);
        IoFile {
            file,
            line: None,
            pos: 0,
        }
    }
}

impl RelIter for IoFile {
    fn pos(&self) -> usize {
        self.pos
    }

    fn advance(&mut self) -> bool {
        let mut buf = String::new();
        match self.file.read_line(&mut buf) {
            Ok(0) => {
                self.line = None;
                false
            }
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                self.line = Some(buf.into());
                true
            }
            Err(e) => false,
        }
    }

    fn row(&mut self) -> Col {
        vec![self.line.clone().unwrap()]
    }
}
