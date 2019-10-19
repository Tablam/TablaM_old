use std::collections::HashSet;
use std::fmt;

use crate::dsl::*;
use crate::types::*;

impl Relation for Table {
    fn shape(&self) -> Shape {
        let (cols, rows) = self.size();

        Shape::Table(cols, rows)
    }

    fn printer(&self) -> RelPrinter<Self>
    where
        Self: Sized,
    {
        RelPrinter::new(self)
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

    fn cross(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                let mut a = self.clone();
                a.schema = self.schema.extend(&schema_it(b.kind()));
                let b = Scalar::repeat(b, self.data.len());
                a.append_column(b);
                a.into()
            }
            Rel::Vector(b) => {
                let schema = self.schema.extend(&schema_it(b.kind()));
                let mut rows = Vec::with_capacity(self.data.len() * b.data.len());

                for a in &self.data {
                    for b in &b.data {
                        let mut row = a.clone();
                        row.push(b.clone());
                        rows.push(row);
                    }
                }
                table_rows(schema, rows).into()
            }
            Rel::Table(b) => {
                let schema = self.schema.extend(&b.schema);
                let mut rows = Vec::with_capacity(self.data.len() * b.data.len());

                for a in &self.data {
                    for b in &b.data {
                        let mut row = a.clone();
                        row.extend_from_slice(&b);
                        rows.push(row);
                    }
                }
                table_rows(schema, rows).into()
            }
            Rel::Seq(_) => self.as_seq().intersect(other),
            Rel::Query(_) => self.as_seq().intersect(other),
        }
    }
}

impl Table {
    pub fn new(schema: Schema, data: Vec<Col>) -> Self {
        Table { schema, data }
    }

    pub fn single(schema: Schema, data: Scalar) -> Self {
        Table {
            schema,
            data: vec![vec![data]],
        }
    }

    pub fn new_cols(schema: Schema, columns: Vec<Col>) -> Self {
        let cols = columns.len();
        let rows = if cols > 0 { columns[0].len() } else { 0 };

        let mut data = Vec::with_capacity(rows);

        for i in 0..rows {
            let mut row = Vec::with_capacity(cols);

            for c in 0..cols {
                row.push(columns[c][i].clone());
            }
            data.push(row);
        }

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

    pub fn as_seq(&self) -> Seq {
        //        Seq::new(
        //            self.schema.clone(),
        //            &self.shape(),
        //            Box::new(self.into_iter()),
        //        )
        unimplemented!()
    }

    pub fn append_column(&mut self, col: Col) {
        for (i, row) in self.data.iter_mut().enumerate() {
            row.push(col[i].clone());
        }
    }
}

impl Iterator for RowsIter<Table> {
    type Item = Col;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < 1 {
            self.pos += 1;
            Some(self.rel.data[self.pos].clone())
        } else {
            None
        }
    }
}

impl IntoIterator for Table {
    type Item = Col;
    type IntoIter = RowsIter<Table>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self)
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

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{:?}", self.schema, self.data)
    }
}

impl fmt::Display for RelPrinter<'_, Table> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[<{}", self.rel.schema)?;
        for row in &self.rel.data {
            writeln!(f, ";")?;
            _print_rows(row, f)?;
        }
        write!(f, ">]")
    }
}
