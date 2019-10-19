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
            Rel::Query(_) => self.as_seq().union(other),
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
            Rel::Query(_) => self.as_seq().diff(other),
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
            Rel::Query(_) => self.as_seq().intersect(other),
        }
    }

    fn cross(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                let schema = schema(&[("col0", self.kind()), ("col1", b.kind())]);

                let a = self.data.clone();
                let b = Scalar::repeat(b, self.data.len());
                table_cols(schema, &vec![a, b]).into()
            }
            Rel::Vector(b) => {
                let schema = schema(&[("col0", self.kind()), ("col1", b.kind())]);
                let mut rows = Vec::with_capacity(self.data.len() * b.data.len());

                for a in &self.data {
                    for b in &b.data {
                        rows.push(vec![a.clone(), b.clone()]);
                    }
                }
                table_rows(schema, rows).into()
            }
            Rel::Table(_) => self.to_table().cross(other),
            Rel::Seq(_) => self.as_seq().cross(other),
            Rel::Query(_) => self.as_seq().cross(other),
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

    pub fn kind(&self) -> DataType {
        self.schema.columns[0].kind
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

    pub fn as_seq(&self) -> Seq {
        //        Seq::new(
        //            self.schema.clone(),
        //            &self.shape(),
        //            Box::new(self.into_iter()),
        //        )
        unimplemented!()
    }

    fn as_set(&self) -> HashSet<Scalar> {
        self.data.iter().cloned().collect()
    }
}

impl Iterator for RowsIter<Vector> {
    type Item = Col;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < 1 {
            self.pos += 1;
            Some(vec![self.rel.data[self.pos].clone()])
        } else {
            None
        }
    }
}

impl IntoIterator for Vector {
    type Item = Col;
    type IntoIter = RowsIter<Vector>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self)
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
