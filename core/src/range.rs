use std::fmt;

use crate::dsl::*;
use crate::types::*;

impl Range {
    pub fn new(start: usize, end: usize, step: usize) -> Self {
        let schema = schema_it(DataType::ISize);
        Range {
            schema,
            start,
            end,
            step,
        }
    }

    fn shape(&self) -> Shape {
        Shape::Table(1, self.end - self.start)
    }

    fn rows(&self) -> RowsIter<Self> {
        RowsIter::new(self.clone())
    }

    fn as_seq(&self) -> Seq {
        Seq::new(self.schema.clone(), &self.shape(), ref_cell(self.rows()))
    }

    fn get(&mut self, pos: usize) -> Option<usize> {
        if pos >= self.start && pos <= self.end {
            Some(pos)
        } else {
            None
        }
    }
}

impl RelIter for RowsIter<Range> {
    fn pos(&self) -> usize {
        self.pos
    }

    fn advance(&mut self) -> bool {
        if self.pos < self.rel.start {
            self.pos = self.rel.start;
        }

        let ok = self.pos < self.rel.end;
        self.pos += self.rel.step;
        ok
    }

    fn row(&mut self) -> Col {
        vec![Scalar::ISize(self.pos as isize)]
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.step != 1 {
            write!(f, "Range({}..{}..{}", self.start, self.end, self.step)?;
        } else {
            write!(f, "Range({}..{}", self.start, self.end)?;
        }
        Ok(())
    }
}
