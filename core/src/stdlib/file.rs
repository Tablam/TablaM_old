use std::io;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use super::super::types::{Rows, Scalar, Col};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FileRead {
    All, Lines, Bytes,
}

pub struct ioFile {
    pub file: File,
    pub read: FileRead,
    pos: u64,
}

//impl Rows for ioFile
//{
//    fn next (&mut self) -> Option<&Scalar> {
//        let mut contents = String::new();
//
//        self.file.read_to_string(&mut contents).unwrap();
//        let data = contents.into();
//        Some(&data)
//    }
//
//    fn row_count(&self) -> usize {1}
//    fn col_count(&self) -> usize {1}
//    fn get_row(&self, pos:usize) -> Option<Col> { Some([].to_vec())}
//}
