#![allow(dead_code)]
#![allow(unused_variables)]

mod core;

use core::types::*;
//use core::operators::*;

fn main() {
    let nums1:Vec<i64> = (0..100).into_iter().collect();
    let nums2:Vec<i64> = (100..200).into_iter().collect();
    let bools1:Vec<bool> = (100..200).into_iter().map( |x| x > 0).collect();

    let f1 = Column::from(nums1);
    let f2 = Column::from(nums2);
    let f3 = Column::from(bools1);
    let s1 = Column::from(vec!(1));

    // println!("Sum:  {:?}", sum_pair(num1, num2));
    println!("Row0:  {:?}", row(&vec!(f1.clone(), f2.clone()), 0));
    println!("Col1:  {:?}", to_i64(&f1));
    println!("Col2:  {:?}", to_i64(&f2));
    println!("Col1 == Col2:  {:?}", &f1 == &f2);
    println!("Sum Dot:  {:?}", f2.clone() + f1.clone());
    println!("Sum Scalar:  {:?}", f2.clone() + s1.clone());
}