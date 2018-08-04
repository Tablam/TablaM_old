//! This module implements the `Cursor` struct, which enables to scan
//! relations in both directions
pub struct Cursor {
    pos: usize,
    row_count: usize,
}

impl Cursor {
    fn new(row_count:usize) -> Self {
        Cursor {
            pos:0,
            row_count,
        }
    }

    fn _set(&mut self, pos:usize) {
        self.pos = pos
    }

    fn eof(&self) -> bool {
        self.pos == self.row_count
    }

    fn first(&mut self) {
        self._set(0)
    }

    fn back(&mut self) -> bool {
        self.skip(-1)
    }

    fn next(&mut self) -> bool {
        self.skip(1)
    }

    fn last(&mut self) {
        let pos = self.row_count;
        self._set(pos)
    }

    fn skip(&mut self, steps:isize) -> bool {
        let pos = (self.pos as isize) + steps;

        if pos < 0 || pos > (self.row_count as isize) {
            return false
        }
        self._set(pos as usize);
        true
    }

    fn pos(&self) -> usize {
        self.pos as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate() {
        let cursor = Cursor::new(0);
        assert_eq!(cursor.eof(), true);

        let mut cursor = Cursor::new(10);

        assert_eq!(cursor.pos, 0);
        assert_eq!(cursor.eof(), false);

        assert_eq!(cursor.next(), true);
        assert_eq!(cursor.next(), true);
        assert_eq!(cursor.pos, 2);

        assert_eq!(cursor.back(), true);
        assert_eq!(cursor.back(), true);
        assert_eq!(cursor.pos, 0);

        cursor.last();
        assert_eq!(cursor.pos, 10);
        assert_eq!(cursor.eof(), true);
        assert_eq!(cursor.next(), false);
        assert_eq!(cursor.pos, 10);

        cursor.first();
        assert_eq!(cursor.eof(), false);
        assert_eq!(cursor.pos, 0);

        assert_eq!(cursor.skip(10), true);
        assert_eq!(cursor.pos, 10);
        assert_eq!(cursor.eof(), true);

        assert_eq!(cursor.skip(-10), true);
        assert_eq!(cursor.pos, 0);
    }
}
