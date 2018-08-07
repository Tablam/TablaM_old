#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use] extern crate lalrpop_util;

// Keep until lalrpop 0.16
lalrpop_mod!(pub tablam);
// pub mod tablam;

mod core;

use core::types::*;
use core::ast::*;
//use core::operators::*;

fn main() {
    let nums1:Vec<i64> = (0..100).into_iter().collect();
    let nums2:Vec<i64> = (100..200).into_iter().collect();
    let nums3:Vec<i64> = (100..200).into_iter().collect();
    let bools1:Vec<bool> = (100..200).into_iter().map( |x| x > 0).collect();

    let f1 = Column::from(nums1);
    let f2 = Column::from(nums2);
    let f3 = Column::from(nums3);
    let f4 = Column::from(bools1);
    let s1 = Column::from(vec!(1));

    // println!("Sum:  {:?}", sum_pair(num1, num2));
    println!("Row0:  {:?}", row(&vec!(f1.clone(), f2.clone()), 0));
    println!("Col1:  {:?}", to_i64(&f1));
    println!("Col2:  {:?}", to_i64(&f2));
    println!("Col3:  {:?}", to_i64(&f3));
    println!("Col1 == Col2:  {:?}", &f1 == &f2);
    println!("Col2 == Col3:  {:?}", &f2 == &f3);
//    println!("Sum Dot:  {:?}", f2.clone() + f1.clone());
//    println!("Sum Scalar:  {:?}", f2.clone() + s1.clone());

    let e = ExpAt {
        line: 5,
        exp: Exp::BinOp(BinOp::Plus,
                        &ExpAt {
                            line: 5,
                            exp: Exp::Scalar(Scalar::I32(3)),
                        },
                        &ExpAt {
                            line: 5,
                            exp: Exp::Scalar(Scalar::I64(4)),
                        })
    };
    println!("e: {:?}", e)
}

#[test]
fn tablam() {
    use tablam::*;

    assert!(
        ScalarLiteralParser::new().parse("12").unwrap()
        == Scalar::I32(12)
        );

    assert!(
        ScalarLiteralParser::new().parse("12i64").unwrap()
        == Scalar::I64(12)
        );

    assert!(
        ScalarLiteralParser::new().parse("12i32").unwrap()
        == Scalar::I32(12)
        );

    assert!(
        ScalarLiteralParser::new().parse("true").unwrap()
        == Scalar::BOOL(true)
        );

    assert!(
        ScalarLiteralParser::new().parse("false").unwrap()
        == Scalar::BOOL(false)
        );

    assert!(
        ScalarLiteralParser::new().parse(r#""hello""#).unwrap()
        == Scalar::UTF8(encode_str("hello"))
        );

    assert!(ScalarLiteralParser::new().parse(r#""hello"#).is_err());
}
