use std::collections::HashSet;
use std::rc::Rc;

use crate::dsl::*;
use crate::types::*;

impl Relation for Vector {
    fn shape(&self) -> Shape {
        Shape::Vector(self.data.len())
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
        let data = self.data.iter().filter(|x| apply(x, &cmp.rhs)).cloned();
        let rel = Self::new(self.schema.clone(), data.collect());
        rel.into()
    }

    fn union(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                let mut data = self.data.clone();
                data.push(b.clone());

                Self::new(self.schema.clone(), data).into()
            }
            Rel::Vector(b) => {
                let data = self.data.iter().chain(b.data.iter()).cloned();
                Self::new(self.schema.clone(), data.collect()).into()
            }
            _ => unimplemented!(),
        }
    }

    fn diff(&self, other: &Rel) -> Rel {
        match other {
            Rel::Vector(b) => {
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
            Rel::Vector(b) => {
                let a = self.as_set();
                let b = b.as_set();

                let result = a.intersection(&b);
                Self::new(self.schema.clone(), result.cloned().collect()).into()
            }
            _ => unimplemented!(),
        }
    }
}

impl Vector {
    pub fn new(schema: Schema, data: Vec<Scalar>) -> Self {
        Vector { schema, data }
    }

    pub fn empty(kind: DataType) -> Self {
        let schema = schema_it(kind);
        let data = vec![];
        Vector { schema, data }
    }

    pub fn new_kind(data: Vec<Scalar>, kind: DataType) -> Self {
        let schema = schema_it(kind);
        Vector { schema, data }
    }

    pub fn new_scalars(data: &[Scalar]) -> Self {
        let kind = data[0].kind();
        let schema = schema_it(kind);
        Vector {
            schema,
            data: data.to_vec(),
        }
    }

    fn as_set(&self) -> HashSet<Scalar> {
        self.data.iter().map(|x| x.clone()).collect()
    }
}

impl RelIter for RowsIter<Vector> {
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
        vec![self.rel.data[pos].clone()]
    }
}
