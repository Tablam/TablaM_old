use std::fmt;

use crate::dsl::{schema, schema_it, table_cols};
use crate::types::*;

impl Default for Scalar {
    fn default() -> Scalar {
        Scalar::None
    }
}

impl Relation for Scalar {
    fn shape(&self) -> Shape {
        Shape::Scalar
    }

    fn printer(&self) -> RelPrinter<Self>
    where
        Self: Sized,
    {
        RelPrinter::new(self)
    }

    fn filter(&self, cmp: CmOp) -> Rel {
        let apply = cmp.get_fn();
        if apply(self, &cmp.rhs) {
            self.clone().into()
        } else {
            self.to_empty_vector().into()
        }
    }

    fn union(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => Vector::new_scalars(&[self.clone(), b.clone()]).into(),
            Rel::Vector(b) => b.append(self).into(),
            Rel::Table(_) => self.to_table().union(other),
            Rel::Seq(_) => self.as_seq().union(other),
            Rel::Query(_) => self.as_seq().union(other),
        }
    }

    fn diff(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                if self == b {
                    Vector::empty(self.kind()).into()
                } else {
                    self.clone().into()
                }
            }
            Rel::Vector(_) => self.to_vector().diff(other),
            Rel::Table(_) => self.to_table().diff(other),
            Rel::Seq(_) => self.as_seq().diff(other),
            Rel::Query(_) => self.as_seq().diff(other),
        }
    }

    fn intersect(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                if self == b {
                    self.clone().into()
                } else {
                    Vector::empty(self.kind()).into()
                }
            }
            Rel::Vector(_) => self.to_vector().intersect(other),
            Rel::Table(_) => self.to_table().intersect(other),
            Rel::Seq(_) => self.as_seq().intersect(other),
            Rel::Query(_) => self.as_seq().diff(other),
        }
    }

    fn cross(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                let schema = schema(&[("col0", self.kind()), ("col1", b.kind())]);
                let a = vec![self.clone()];
                let b = vec![b.clone()];
                table_cols(schema, &vec![a, b]).into()
            }
            Rel::Vector(_) => self.to_vector().cross(other),
            Rel::Table(_) => self.to_table().cross(other),
            Rel::Seq(_) => self.as_seq().cross(other),
            Rel::Query(_) => self.as_seq().cross(other),
        }
    }
}

impl Scalar {
    pub fn to_empty_vector(&self) -> Vector {
        Vector::new_kind(vec![], self.kind())
    }

    pub fn to_vector(&self) -> Vector {
        Vector::new_scalars(&[self.clone()])
    }

    pub fn to_table(&self) -> Table {
        Table::single(schema_it(self.kind()), self.clone())
    }

    pub fn repeat(of: &Scalar, times: usize) -> Vec<Scalar> {
        vec![of.clone(); times]
    }

    pub fn kind(&self) -> DataType {
        match self {
            Scalar::None => DataType::None,
            Scalar::Bool(_) => DataType::Bool,
            Scalar::ISize(_) => DataType::ISize,
            Scalar::I32(_) => DataType::I32,
            Scalar::I64(_) => DataType::I64,
            Scalar::F64(_) => DataType::F64,
            Scalar::Decimal(_) => DataType::Decimal,
            Scalar::DateTime(_) => DataType::DateTime,
            Scalar::UTF8(_) => DataType::UTF8,
            Scalar::Rel(_) => DataType::Rel,
        }
    }

    pub fn as_seq(&self) -> Seq {
        //        Seq::new(
        //            schema_it(self.kind()),
        //            &self.shape(),
        //            Box::new(self.into_iter()),
        //        )
        unimplemented!()
    }
}

impl Iterator for RowsIter<Scalar> {
    type Item = Col;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < 1 {
            self.pos += 1;
            Some(vec![self.rel.clone()])
        } else {
            None
        }
    }
}

impl IntoIterator for Scalar {
    type Item = Col;
    type IntoIter = RowsIter<Scalar>;

    fn into_iter(self) -> Self::IntoIter {
        RowsIter::new(self)
    }
}

impl RelIter for RowsIter<Scalar> {
    fn pos(&self) -> usize {
        self.pos
    }

    fn advance(&mut self) -> bool {
        let ok = self.pos < 1;
        self.pos += 1;
        ok
    }

    fn row(&mut self) -> Col {
        vec![self.rel.clone()]
    }
}

pub fn bin_op<T, Op>(op: Op, x: T, y: T) -> Scalar
where
    Op: FnOnce(T, T) -> T,
    T: From<Scalar>,
    Scalar: From<T>,
{
    op(x, y).into()
}

pub fn bin_op_by<T, Op>(op: Op, x: Scalar, y: Scalar) -> Scalar
where
    Op: FnOnce(T, T) -> T,
    T: From<Scalar>,
    Scalar: From<T>,
{
    bin_op(op, x.into(), y.into())
}

impl fmt::Display for Scalar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scalar::None => write!(f, "None"),
            Scalar::Bool(x) => write!(f, "{}", x),
            Scalar::ISize(x) => write!(f, "{}", x),
            Scalar::I32(x) => write!(f, "{}", x),
            Scalar::I64(x) => write!(f, "{}", x),
            Scalar::F64(x) => write!(f, "{}", x),
            Scalar::Decimal(x) => write!(f, "{}", x),
            Scalar::DateTime(x) => write!(f, "{}", x),
            Scalar::UTF8(x) => write!(f, "{}", x),
            Scalar::Rel(x) => write!(f, "{}", x),
        }
    }
}

impl fmt::Display for RelPrinter<'_, Scalar> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[it:{};", self.rel.kind())?;
        write!(f, "{}]", self.rel)
    }
}
