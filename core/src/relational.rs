use crate::types::*;

impl Relation for Rel {
    fn shape(&self) -> Shape {
        match self {
            Rel::One(x) => x.shape(),
            Rel::Vector(x) => x.shape(),
            //Rel::Table(x) => x.shape(),
            _ => unimplemented!(),
        }
    }

    fn rows(&self) -> RowsIter<Self>
    where
        Self: Sized,
    {
        RowsIter::new(self.clone())
    }

    fn as_seq(&self) -> Seq {
        match self {
            Rel::One(x) => x.as_seq(),
            Rel::Vector(x) => x.as_seq(),
            Rel::Range(x) => x.as_seq(),
            Rel::Seq(x) => x.as_seq(),
            Rel::Table(x) => x.as_seq(),
        }
    }

    fn filter(&self, cmp: CmOp) -> Rel {
        match self {
            Rel::One(x) => x.filter(cmp),
            Rel::Vector(x) => x.filter(cmp),
            Rel::Range(x) => x.filter(cmp),
            Rel::Seq(x) => x.filter(cmp),
            Rel::Table(x) => x.filter(cmp),
        }
    }

    fn union(&self, other: &Rel) -> Rel {
        match self {
            Rel::One(x) => x.union(other),
            Rel::Vector(x) => x.union(other),
            Rel::Range(x) => x.union(other),
            Rel::Seq(x) => x.union(other),
            Rel::Table(x) => x.union(other),
        }
    }

    fn diff(&self, other: &Rel) -> Rel {
        match self {
            Rel::One(x) => x.diff(other),
            Rel::Vector(x) => x.diff(other),
            Rel::Range(x) => x.diff(other),
            Rel::Seq(x) => x.diff(other),
            Rel::Table(x) => x.diff(other),
        }
    }

    fn intersect(&self, other: &Rel) -> Rel {
        match self {
            Rel::One(x) => x.intersect(other),
            Rel::Vector(x) => x.intersect(other),
            Rel::Range(x) => x.intersect(other),
            Rel::Seq(x) => x.intersect(other),
            Rel::Table(x) => x.intersect(other),
        }
    }
}

impl Rel {
    pub fn query(self, query: &[Query]) -> Rel {
        if query.is_empty() {
            self.clone()
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
                    _ => unimplemented!(),
                };
            }
            next
        }
    }
}
