use std::fmt;

use crate::dsl::schema_it;
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

    fn rows(&self) -> RowsIter<Self>
    where
        Self: Sized,
    {
        RowsIter::new(self.clone())
    }

    fn as_seq(&self) -> Seq {
        Seq::new(schema_it(self.kind()), &self.shape(), ref_cell(self.rows()))
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
            Rel::Table(_) => Table::single(schema_it(self.kind()), self.clone()).union(other),
            _ => unimplemented!(),
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
            _ => unimplemented!(),
        }
    }
    fn intersect(&self, other: &Rel) -> Rel {
        match other {
            Rel::One(b) => {
                if self != b {
                    self.clone().into()
                } else {
                    Vector::empty(self.kind()).into()
                }
            }
            _ => unimplemented!(),
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
