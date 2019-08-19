use std::cell::RefCell;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::types::*;

impl Relation for Seq {
    fn shape(&self) -> Shape {
        self.shape
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
        self.clone()
    }

    fn filter(&self, cmp: CmOp) -> Rel {
        let iter = FilterIter {
            cmp,
            iter: Rc::new(self.rows()),
        };

        Seq::new(self.schema.clone(), &self.shape, ref_cell(iter)).into()
    }

    fn union(&self, other: &Rel) -> Rel {
        match other {
            Rel::Seq(b) => {
                Self::of_union(&self.schema, &self.shape, self.iter.clone(), b.iter.clone()).into()
            }
            _ => unimplemented!(),
        }
    }

    fn diff(&self, other: &Rel) -> Rel {
        unimplemented!()
    }
    fn intersect(&self, other: &Rel) -> Rel {
        unimplemented!()
    }
}

impl Seq {
    pub fn new(schema: Schema, shape: &Shape, iter: Rc<RefCell<dyn RelIter>>) -> Self {
        Seq {
            iter,
            schema,
            shape: *shape,
        }
    }

    pub fn of_union(
        schema: &Schema,
        shape: &Shape,
        lhs: Rc<RefCell<dyn RelIter>>,
        rhs: Rc<RefCell<dyn RelIter>>,
    ) -> Self {
        let iter = UnionIter {
            lhs,
            rhs,
            first: true,
        };
        Self::new(schema.clone(), shape, ref_cell(iter))
    }

    pub fn materialize(&mut self) -> Rel {
        let mut b = self.iter.borrow_mut();

        match self.shape {
            Shape::Vector(rows) => {
                let mut rows = Vec::with_capacity(rows);
                while let Some(row) = b.next() {
                    rows.push(row[0].clone());
                }

                Vector::new(self.schema.clone(), rows).into()
            }
            _ => unimplemented!(),
        }
    }
}

impl RelIter for RowsIter<Seq> {
    fn pos(&self) -> usize {
        let iter = self.rel.iter.borrow();
        iter.pos()
    }

    fn advance(&mut self) -> bool {
        let mut iter = self.rel.iter.borrow_mut();
        iter.advance()
    }

    fn row(&mut self) -> Col {
        let mut iter = self.rel.iter.borrow_mut();
        iter.row()
    }
}

struct UnionIter {
    pub first: bool,
    pub lhs: Rc<RefCell<dyn RelIter>>,
    pub rhs: Rc<RefCell<dyn RelIter>>,
}

impl RelIter for UnionIter {
    fn pos(&self) -> usize {
        if self.first {
            self.lhs.borrow().pos()
        } else {
            self.rhs.borrow().pos()
        }
    }

    fn advance(&mut self) -> bool {
        let mut a = self.lhs.borrow_mut();
        let mut b = self.rhs.borrow_mut();

        if self.first {
            if a.advance() {
                true
            } else {
                self.first = false;
                b.advance()
            }
        } else {
            b.advance()
        }
    }

    fn row(&mut self) -> Col {
        if self.first {
            let mut a = self.lhs.borrow_mut();
            a.row()
        } else {
            let mut b = self.rhs.borrow_mut();
            b.row()
        }
    }
}

struct FilterIter {
    pub cmp: CmOp,
    pub iter: Rc<dyn RelIter>,
}

impl RelIter for FilterIter {
    fn pos(&self) -> usize {
        self.iter.pos()
    }

    fn advance(&mut self) -> bool {
        let iter = std::rc::Rc::get_mut(&mut self.iter).unwrap();
        let apply = self.cmp.get_fn();

        while iter.advance() {
            let value = &iter.row()[self.cmp.lhs];
            if apply(value, &self.cmp.rhs) {
                return true;
            }
        }
        false
    }

    fn row(&mut self) -> Col {
        let iter = std::rc::Rc::get_mut(&mut self.iter).unwrap();
        iter.row()
    }
}

impl fmt::Debug for Seq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Seq({:?} of {:?})", self.schema, self.shape)
    }
}

impl fmt::Display for Seq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Seq({} of {:?})", self.schema, self.shape)?;
        Ok(())
    }
}

impl Hash for Seq {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.schema.hash(state);
        self.shape.hash(state);
    }
}

impl PartialEq for Seq {
    fn eq(&self, other: &Seq) -> bool {
        self.schema == other.schema && self.shape == other.shape
    }
}

impl Eq for Seq {}

impl PartialOrd for Seq {
    fn partial_cmp(&self, other: &Seq) -> Option<Ordering> {
        let a = self.shape.partial_cmp(&other.shape);
        let b = self.schema.partial_cmp(&other.schema);
        a.partial_cmp(&b)
    }
}

impl Ord for Seq {
    fn cmp(&self, other: &Seq) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Clone for Seq {
    fn clone(&self) -> Self {
        Seq::new(self.schema.clone(), &self.shape, self.iter.clone())
    }
}

impl fmt::Display for RelPrinter<'_, Seq> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{};", self.rel.schema);
        write!(f, "{};", self.rel);
        write!(f, "]")
    }
}
