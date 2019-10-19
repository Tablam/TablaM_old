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

    fn as_seq(&self) -> Seq {
        unimplemented!()
        //        Seq::new(
        //            self.schema.clone(),
        //            &self.shape(),
        //            Box::new(self.into_iter()),
        //        )
    }
}

impl IntoIterator for Range {
    type Item = Col;
    type IntoIter = RowsIter<Range>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self)
    }
}

impl Iterator for RowsIter<Range> {
    type Item = Col;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.rel.start && self.pos <= self.rel.end {
            let pos = Scalar::ISize(self.pos as isize);
            self.pos += self.rel.step;
            Some(vec![pos])
        } else {
            None
        }
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
