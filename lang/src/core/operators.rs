#![allow(unused_imports)]
use std::rc::Rc;
use std::ops::{Add};

extern crate bit_vec;
use self::bit_vec::BitVec;

use super::types::*;

/// The ops in TablaM are columnar, and follow this pattern
/// [1, 2, 3] +  [1, 2, 3] = [2, 4, 6]
/// [1, 2, 3] +  1 = [1, 3, 4]
/// 1 + [1, 2, 3] = [1, 3, 4]
/// [1, 2, 3] +  [1, 2] = ERROR must be same length

impl Add for Scalar {
    type Output = Scalar;

    fn add(self, other: Scalar) -> Scalar {
        use self::Scalar::*;
        match (self, other) {
            (I32(x), I32(y)) => I32(x + y),
            (I64(x), I64(y)) => I64(x + y),
            (I64(x), I64(y)) => I64(x + y),
            // (UTF8(s), UTF8(o)) => UTF8(s+o),
            p => panic!("Not implemented: cannot add {:?}", p),
        }
    }
}

// TODO: The operators follow this patterns:
// maps: ColumnExp & ColumnExp = Column (+ [1, 2] [2, 3] = [3, 6])
// reduce: ColumnExp = Column (+[1, 2] = 3)

struct Cmp<'a, F>
    where
        F:  Fn(&Scalar, &Scalar) -> bool
{
    left:   &'a Data,
    right:  &'a Data,
    pos  :  usize,
    apply:  F,
}

impl <'a, F> Iterator for Cmp<'a, F>
    where
        F:  Fn(&Scalar, &Scalar) -> bool
{
    type Item = (usize, bool);

    fn next(&mut self) -> Option<(usize, bool)> {
        if self.pos < self.left.len {
            let l = &self.left.data[self.pos];
            let r = &self.right.data[self.pos];
            let result = (self.pos, (self.apply)(l, r));

            self.pos +=1;
            return Some(result)
        };
        None
    }
}

fn scan_simple<'a, F>(left: &'a Data, right: &'a Data, apply: &'a F ) -> impl Iterator<Item = bool> + 'a
    where
        F:  Fn(&Scalar, &Scalar) -> bool
{
    let value = &right.data[0];

    let scan = left.data.iter()
        .map(move |x|  {
            let r:bool = apply(x, value);
            r
        });
    scan
}

fn scan_both<'a, F>(left: &'a Data, right: &'a Data, apply: &'a F ) -> impl Iterator<Item = bool> + 'a
    where
        F:  Fn(&Scalar, &Scalar) -> bool
{
    let scan = left.data.iter()
        .zip(right.data.iter())
        .map(move |(x, y)|  {
            let r:bool = apply(x, y);
            r
        });
    scan
}

fn eq(left:&Data, right:&Data) -> Data
{
    let result:Vec<bool> = scan_both(&left, &right, &PartialEq::eq).collect();

    Data::from(result)
}

fn filter(left:&Data, right:&Data) -> Data
{
    let rows = left.clone();
    let result=
        if right.len == 1 {
            let cmp = scan_simple(&left, &right, &PartialEq::eq);
            rows.data.iter()
                .zip(cmp)
                .filter_map(|(x, check)| {
                    if check {
                        Some(x.clone())
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            let cmp = scan_simple(&left, &right, &PartialEq::eq);
            rows.data.iter()
                .zip(cmp)
                .filter_map(|(x, check)| {
                    if check {
                        Some(x.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

    Data::new(result, rows.kind)
}

fn filter_pos(left:&Data, right:&Data) -> Vec<usize>
{
    let result:Vec<usize>=
        if right.len == 1 {
            let cmp = scan_simple(&left, &right, &PartialEq::eq);
            cmp.enumerate()
                .filter_map(|(x, check)| {
                    if check {
                        Some(x)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            let cmp = scan_simple(&left, &right, &PartialEq::eq);
            cmp.enumerate()
                .filter_map(|(x, check)| {
                    if check {
                        Some(x)
                    } else {
                        None
                    }
                })
                .collect()
        };

    result
}

struct Join
{
    left: Data,
    right:Data,
    pos_x:usize,
    pos_y:usize,
}

impl Iterator for Join {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        let start_x = self.pos_x;
        let start_y = self.pos_y;

        for x in start_x..self.left.len {
            for y in start_y..self.right.len {
                // println!("{} {:?} == {} {:?}", x, &self.left.data[x], y , &self.right.data[y]);
                if &self.left.data[x] == &self.right.data[y] {
                    let result = Some((x, y));

                    self.pos_x = x;
                    self.pos_y = y + 1;

                    return result
                }
            }
        };
        None
    }
}

fn nested_loop(left:Data, right:Data) -> impl Iterator<Item = (usize, usize)>
{
    Join
        {
            left,
            right,
            pos_x:0,
            pos_y:0,
        }.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nums1() -> Vec<i64> {
        vec![1, 2, 3]
    }

    fn make_nums2() -> Vec<i64> {
        vec![4, 2, 3]
    }

    fn col(pos:usize) -> ColumnExp {
        ColumnExp::Pos(pos)
    }

    fn columns() -> (ColumnExp, ColumnExp) {
        let pick1 = ColumnExp::Name("col0".to_string());
        let pick2 = col(1);

        (pick1, pick2)
    }

//    fn make_rel1() -> Rc<Frame> {
//        let nums1 = make_nums1();
//        let nums2 = make_nums2();
//
//        let col1 = Column::from(nums1);
//        let col2 = Column::from(nums2);
//
//        Rc::new(Frame::new(vec!(col1, col2)))
//    }
//
//    #[test]
//    fn test_select() {
//        let nums1 = make_nums1();
//        let f1 = make_rel1();
//
//        let (pick1, pick2) = columns();
//
//        let col3 = select(f1.clone(), &pick1);
//        let nums3:Vec<i64> = col3.as_slice().into();
//        assert_eq!(nums1, nums3);
//
//        let cols = deselect(f1.clone(), &pick1);
//        assert_eq!(cols.len(), 1);
//    }
//
//    #[test]
//    fn test_compare() {
//        let f1 = make_rel1();
//        let (pick1, pick2) = columns();
//
//        let filter_eq = CompareRel::eq(f1.clone(), pick1, pick2);
//        let filter_not_eq = CompareRel::noteq(f1.clone(), col(0), col(1));
//
//        let result_eq = drain_vec(&compare(filter_eq));
//        println!("= {:?}", result_eq);
//        assert_eq!(result_eq, [false, true, true]);
//
//        let result_not_eq =  drain_vec(&compare(filter_not_eq));
//        println!("<> {:?}", result_eq);
//        assert_eq!(result_not_eq, [true, false, false]);
//    }
//
//    #[test]
//    fn test_filter() {
//        let f1 = make_rel1();
//        let (pick1, pick2) = columns();
//
//        let filter_eq = CompareRel::eq(f1.clone(), pick1, pick2);
//        let filter_not_eq = CompareRel::noteq(f1.clone(), col(0), col(1));
//
//        let result_eq = filter(filter_eq);
//        println!("= {:?}", result_eq);
////        assert_eq!(result_eq, [false, true, true]);
//
//        let result_not_eq =   filter(filter_not_eq);
//        println!("<> {:?}", result_eq);
////        assert_eq!(result_not_eq, [true, false, false]);
//    }
////    #[test]
////    fn math() {
////
////    }
}
