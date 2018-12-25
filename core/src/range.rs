use std::fmt;

use super::types::*;

impl Range {
    pub fn new(start:isize, end:isize, step:usize) -> Self {
        Range {
            start,
            end,
            step
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.step != 1 {
            write!(f, "Range({}..{}", self.start, self.end)?;
        } else {
            write!(f, "Range({}..{}..{}", self.start, self.end, self.step)?;
        }
        Ok(())
    }
}