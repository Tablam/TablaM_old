use std::collections::HashSet;

use crate::dsl::*;
use crate::types::*;

impl Relation for Table {
    fn shape(&self) -> Shape {
        let (cols, rows) = self.size();

        Shape::Table(cols, rows)
    }

    fn rows(&self) -> RowsIter<Self>
    where
        Self: Sized,
    {
        RowsIter::new(self.clone())
    }

    fn as_seq(&self) -> Seq {
        Seq::new(self.schema.clone(), &self.shape(), ref_cell(self.rows()))
    }

    fn filter(&self, cmp: CmOp) -> Rel {
        let apply = cmp.get_fn();
        let data = self
            .data
            .iter()
            .filter(|x| apply(&x[cmp.lhs], &cmp.rhs))
            .cloned();
        let rel = Self::new(self.schema.clone(), data.collect());
        rel.into()
    }

    fn union(&self, other: &Rel) -> Rel {
        match other {
            Rel::Table(b) => {
                let data = self.data.iter().chain(b.data.iter()).cloned();
                Self::new(self.schema.clone(), data.collect()).into()
            }
            _ => unimplemented!(),
        }
    }

    fn diff(&self, other: &Rel) -> Rel {
        match other {
            Rel::Table(b) => {
                let a = self.as_set();
                let b = b.as_set();

                let result = a.difference(&b);
                Self::new(self.schema.clone(), result.cloned().collect()).into()
            }
            _ => unimplemented!(),
        }
    }

    fn intersect(&self, other: &Rel) -> Rel {
        match other {
            Rel::Table(b) => {
                let a = self.as_set();
                let b = b.as_set();

                let result = a.intersection(&b);
                Self::new(self.schema.clone(), result.cloned().collect()).into()
            }
            _ => unimplemented!(),
        }
    }
}

impl Table {
    pub fn new(schema: Schema, data: Vec<Col>) -> Self {
        Table { schema, data }
    }

    pub fn empty(kind: DataType) -> Self {
        let schema = schema_it(kind);
        let data = vec![];
        Table { schema, data }
    }

    fn size(&self) -> (usize, usize) {
        let rows = self.data.len();

        let cols = if rows > 0 { self.data[0].len() } else { 0 };

        (cols, rows)
    }

    fn as_set(&self) -> HashSet<Col> {
        self.data.iter().cloned().collect()
    }
}

impl RelIter for RowsIter<Table> {
    fn pos(&self) -> usize {
        self.pos
    }

    fn advance(&mut self) -> bool {
        let ok = self.pos < self.rel.data.len();
        self.pos += 1;
        ok
    }

    fn row(&mut self) -> Col {
        let pos = self.pos - 1;
        self.rel.data[pos].clone()
    }
}
