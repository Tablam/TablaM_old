use std::collections::HashSet;
use std::fmt;

use crate::dsl::*;
use crate::types::*;

impl Relation for Vector {
    fn shape(&self) -> Shape {
        Shape::Vector(self.data.len())
    }

    fn printer(&self) -> RelPrinter<Self>
    where
        Self: Sized,
    {
        RelPrinter::new(self)
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
            Rel::One(b) => self.append(b).into(),
            Rel::Vector(b) => {
                let data = self.data.iter().chain(b.data.iter()).cloned();
                Self::new(self.schema.clone(), data.collect()).into()
            }
            Rel::Table(_) => self.to_table().union(other),
            Rel::Seq(_) => self.as_seq().union(other),
        }
    }

    fn diff(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                if self.data.contains(b) {
                    Self::empty(self.schema[0].kind).into()
                } else {
                    b.clone().into()
                }
            }
            Rel::Vector(b) => {
                let a = self.as_set();
                let b = b.as_set();

                let result = a.difference(&b);
                Self::new(self.schema.clone(), result.cloned().collect()).into()
            }
            Rel::Table(_) => self.to_table().diff(other),
            Rel::Seq(_) => self.as_seq().diff(other),
        }
    }

    fn intersect(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                if self.data.contains(b) {
                    b.clone().into()
                } else {
                    Self::empty(self.schema[0].kind).into()
                }
            }
            Rel::Vector(b) => {
                let a = self.as_set();
                let b = b.as_set();

                let result = a.intersection(&b);
                Self::new(self.schema.clone(), result.cloned().collect()).into()
            }
            Rel::Table(_) => self.to_table().intersect(other),
            Rel::Seq(_) => self.as_seq().intersect(other),
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

    pub fn to_table(&self) -> Table {
        let mut data = Vec::with_capacity(self.data.len());
        for row in &self.data {
            data.push(vec![row.clone()]);
        }
        Table::new(self.schema.clone(), data)
    }

    pub fn append(&self, value: &Scalar) -> Self {
        let mut data = self.data.clone();
        data.push(value.clone());

        Self::new(self.schema.clone(), data)
    }

    fn as_set(&self) -> HashSet<Scalar> {
        self.data.iter().cloned().collect()
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

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{:?}", self.schema, self.data)
    }
}

impl fmt::Display for RelPrinter<'_, Vector> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{};", self.rel.schema)?;
        _print_rows(&self.rel.data, f)?;
        write!(f, "]")
    }
}
