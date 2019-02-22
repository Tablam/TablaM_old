use std::fmt;
use std::hash::{Hash, Hasher};

use super::types::*;

impl fmt::Debug for Box<&RelOp> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Generator {:p}", self)
    }
}

impl PartialEq for Box<&RelOp> {
    fn eq(&self, other: &Box<&RelOp>) -> bool {
        self == other
    }
}

impl PartialOrd for Box<&RelOp> {
    fn partial_cmp(&self, other: &Box<&RelOp> ) -> Option< std::cmp::Ordering> {
        if self == other {
            return Some(std::cmp::Ordering::Equal);
        }

        None
    }
}
impl Ord for Box<&RelOp> {
    fn cmp(&self, _other: &Box<&RelOp>) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}

impl Eq for Box<&RelOp> {}

impl Hash for Box<&RelOp> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let address = format!("{:p}", self);
        address.hash(state);
    }
}
