use std::fmt;
use std::rc::Rc;

use crate::types::*;

impl Relation for QueryIter {
    fn shape(&self) -> Shape {
        self.rel.shape()
    }

    fn printer(&self) -> RelPrinter<Self>
    where
        Self: Sized,
    {
        RelPrinter::new(self)
    }

    fn filter(&self, cmp: CmOp) -> Rel {
        self.push_query(Query::Where(cmp)).into()
    }

    fn union(&self, other: &Rel) -> Rel {
        //        self.push_query(Query::union(other.clone())).into()
        unimplemented!()
    }

    fn diff(&self, other: &Rel) -> Rel {
        //        self.push_query(Query::diff(other.clone())).into()
        unimplemented!()
    }

    fn intersect(&self, other: &Rel) -> Rel {
        //        self.push_query(Query::intersection(Rc::new(other))).into()
        unimplemented!()
    }

    fn cross(&self, other: &Rel) -> Rel {
        //        self.push_query(Query::cross(other.clone())).into()
        unimplemented!()
    }
}

impl QueryIter {
    pub fn new(rel: Rc<Rel>, query: Vec<Query>) -> Self {
        QueryIter { rel, query }
    }

    pub fn push_query(&self, query: Query) -> Self {
        let mut q = self.query.clone();
        q.push(query);
        QueryIter::new(self.rel.clone(), q)
    }

    pub fn as_seq(&self) -> Seq {
        unimplemented!()
    }

    pub fn materialize(self) -> Rc<Rel> {
        match self.rel.as_ref() {
            Rel::Query(q) => q.clone().materialize(),
            _ => {
                let mut next = self.rel;
                for q in &self.query {
                    let rel = match q {
                        Query::Where(filter) => next.filter(filter.clone()),
                        Query::Set(query, other) => match query {
                            SetQuery::Union => next.union(&other),
                            SetQuery::Diff => next.diff(&other),
                            SetQuery::Intersection => next.intersect(&other),
                        },
                        Query::Join(join, other) => match join {
                            Join::Cross => next.cross(&other),
                            _ => unimplemented!(),
                        },
                        _ => unimplemented!(),
                    };
                    next = Rc::new(rel)
                }
                next
            }
        }
    }
}

impl fmt::Display for QueryIter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[Query;")?;
        write!(f, "{:?};", self.rel)?;
        write!(f, "]")
    }
}

impl fmt::Display for RelPrinter<'_, QueryIter> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Query;")?;
        write!(f, "{:?};", self.rel.query)?;
        write!(f, "]")
    }
}
