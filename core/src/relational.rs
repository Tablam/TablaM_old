use std::fmt;

use crate::types::*;

impl Relation for Rel {
    fn shape(&self) -> Shape {
        match self {
            Rel::One(x) => x.shape(),
            Rel::Vector(x) => x.shape(),
            Rel::Table(x) => x.shape(),
            Rel::Seq(x) => x.shape(),
            Rel::Query(x) => x.shape(),
        }
    }

    fn printer(&self) -> RelPrinter<Self>
    where
        Self: Sized,
    {
        RelPrinter::new(self)
    }

    fn filter(&self, cmp: CmOp) -> Rel {
        match self {
            Rel::One(x) => x.filter(cmp),
            Rel::Vector(x) => x.filter(cmp),
            Rel::Seq(x) => x.filter(cmp),
            Rel::Table(x) => x.filter(cmp),
            Rel::Query(x) => x.filter(cmp),
        }
    }

    fn union(&self, other: &Rel) -> Rel {
        match self {
            Rel::One(x) => x.union(other),
            Rel::Vector(x) => x.union(other),
            Rel::Seq(x) => x.union(other),
            Rel::Table(x) => x.union(other),
            Rel::Query(x) => x.union(other),
        }
    }

    fn diff(&self, other: &Rel) -> Rel {
        match self {
            Rel::One(x) => x.diff(other),
            Rel::Vector(x) => x.diff(other),
            Rel::Seq(x) => x.diff(other),
            Rel::Table(x) => x.diff(other),
            Rel::Query(x) => x.diff(other),
        }
    }

    fn intersect(&self, other: &Rel) -> Rel {
        match self {
            Rel::One(x) => x.intersect(other),
            Rel::Vector(x) => x.intersect(other),
            Rel::Seq(x) => x.intersect(other),
            Rel::Table(x) => x.intersect(other),
            Rel::Query(x) => x.intersect(other),
        }
    }

    fn cross(&self, other: &Rel) -> Rel {
        match self {
            Rel::One(x) => x.cross(other),
            Rel::Vector(x) => x.cross(other),
            Rel::Seq(x) => x.cross(other),
            Rel::Table(x) => x.cross(other),
            Rel::Query(x) => x.cross(other),
        }
    }
}

impl Rel {
    fn as_seq(&self) -> Seq {
        match self {
            Rel::One(x) => x.as_seq(),
            Rel::Vector(x) => x.as_seq(),
            Rel::Seq(x) => unreachable!(),
            Rel::Table(x) => x.as_seq(),
            Rel::Query(x) => x.as_seq(),
        }
    }

    pub fn query(self, query: &[Query]) -> Rel {
        if query.is_empty() {
            self
        } else {
            let mut next = self;
            for q in query {
                next = match q {
                    Query::Where(filter) => next.filter(filter.clone()),
                    Query::Set(query, other) => match query {
                        SetQuery::Union => next.union(&other),
                        SetQuery::Diff => next.diff(&other),
                        SetQuery::Intersection => next.intersect(&other),
                    },
                    //Query::Sort(asc, pos) => next.sorted(*asc, *pos),
                    Query::Join(join, other) => match join {
                        Join::Cross => next.cross(&other),
                        _ => unimplemented!(),
                    },
                    _ => unimplemented!(),
                };
            }
            next
        }
    }
}

impl fmt::Display for Rel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rel::One(x) => write!(f, "{}", x),
            Rel::Seq(x) => write!(f, "{}", x),
            Rel::Vector(x) => write!(f, "{}", x),
            Rel::Table(x) => write!(f, "{}", x),
            Rel::Query(x) => write!(f, "{}", x),
        }
    }
}

impl fmt::Display for RelPrinter<'_, Rel> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.rel {
            Rel::One(x) => write!(f, "{}", x.printer()),
            Rel::Seq(x) => write!(f, "{}", x.printer()),
            Rel::Vector(x) => write!(f, "{}", x.printer()),
            Rel::Table(x) => write!(f, "{}", x.printer()),
            Rel::Query(x) => write!(f, "{}", x.printer()),
        }
    }
}
