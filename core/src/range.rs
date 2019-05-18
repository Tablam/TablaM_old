use std::fmt;
use std::hash::{Hash, Hasher};
use super::types::*;

impl Range {
    pub fn new(start:isize, end:isize, step:isize) -> Self {
        Range {
            start,
            end,
            step,
            buffer: vec![Scalar::None]
        }
    }

    fn get(&mut self, pos:isize) -> Option<isize> {
        if pos >= self.start && pos <= self.end {
            Some(pos)
        } else {
            None
        }
    }
}

impl Buffered for Range {
    fn buffer(&mut self) -> &mut [Scalar] {
        self.buffer.as_mut_slice()
    }

    fn read_from_buffer(&mut self, pos:usize) -> Option<&Scalar> {
        if let Some(x) = self.get(pos as isize) {
            let value = &Scalar::ISize(x as isize);
            let data = vec![value];
            self.fill(&data);
            Some(&self.buffer[0])
        } else {
            None
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

impl Hash for Range {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
        self.step.hash(state);
    }
}