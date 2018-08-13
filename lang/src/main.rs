#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use] extern crate lalrpop_util;

// Keep until lalrpop 0.16
lalrpop_mod!(pub tablam);
// pub mod tablam;

mod core;

use core::types::*;
use core::ast::*;
use core::ast::ColumnExp;
use std::io::{self, Write, BufRead};
//use core::operators::*;

fn main() -> io::Result<()> {
    // let nums1:Vec<i64> = (0..100).into_iter().collect();
    // let nums2:Vec<i64> = (100..200).into_iter().collect();
    // let nums3:Vec<i64> = (100..200).into_iter().collect();
    // let bools1:Vec<bool> = (100..200).into_iter().map( |x| x > 0).collect();

    // let f1 = Column::from(nums1);
    // let f2 = Column::from(nums2);
    // let f3 = Column::from(nums3);
    // let f4 = Column::from(bools1);
    // let s1 = Column::from(vec!(1));

    // // println!("Sum:  {:?}", sum_pair(num1, num2));
    // println!("Row0:  {:?}", row(&vec!(f1.clone(), f2.clone()), 0));
    // println!("Col1:  {:?}", to_i64(&f1));
    // println!("Col2:  {:?}", to_i64(&f2));
    // println!("Col3:  {:?}", to_i64(&f3));
    // println!("Col1 == Col2:  {:?}", &f1 == &f2);
    // println!("Col2 == Col3:  {:?}", &f2 == &f3);
//  //   println!("Sum Dot:  {:?}", f2.clone() + f1.clone());
//  //   println!("Sum Scalar:  {:?}", f2.clone() + s1.clone());

    // let e = Exp::BinOp(
    //     BinOp::Plus,
    //     Exp::Scalar(Scalar::I32(3)).into(),
    //     Exp::Scalar(Scalar::I64(4)).into()
    //     );
    // println!("e: {:?}", e)

    // let mut buffer = String::new();
    loop {
        use tablam::*;

        // io::stdin().read_to_string(&mut buffer)?;
        let stdin = io::stdin();
        print!("> ");
        std::io::stdout().flush().unwrap();
        for line in stdin.lock().lines() {
            let parser = StatementParser::new();
            match parser.parse(&line.unwrap()) {
                Ok(ast) => println!("ok: {:?}", ast),
                Err(err) => println!("error: {:?}", err),
            }
            print!("> ");
            std::io::stdout().flush().unwrap();
        }
    }
}

#[test]
fn tablam() {
    use tablam::*;

    assert!(
        ScalarLiteralParser::new().parse("12").unwrap()
        == Scalar::I32(12)
        );

    assert!(
        ExprParser::new().parse("true").unwrap()
        == Exp::Name("true".into())
        );

    assert!(
        ExprParser::new().parse("false").unwrap()
        == Exp::Name("false".into())
        );

    assert!(
        ScalarLiteralParser::new().parse(r#""hello""#).unwrap()
        == Scalar::UTF8(encode_str("hello"))
        );

    assert!(ScalarLiteralParser::new().parse(r#""hello"#).is_err());

    assert!(ScalarLiteralParser::new().parse("1").unwrap()
            == Scalar::I32(1));

    assert!(ExprParser::new().parse("1 + 2").unwrap()
            == Exp::BinOp(BinOp::Plus,
                          Exp::Scalar(Scalar::I32(1)).into(),
                          Exp::Scalar(Scalar::I32(2)).into()));

    assert!(ColumnLiteralParser::new().parse("[name:string; \"hello\" \"world\"]").unwrap()
            ==
            ColumnExp {
                name: Some("name".into()),
                ty: Some("string".into()),
                es: vec!(
                    Exp::Scalar(Scalar::UTF8("hello".into())),
                    Exp::Scalar(Scalar::UTF8("world".into())),
                    )
            });

    assert!(ColumnLiteralParser::new().parse("[:string; \"hello\" \"world\"]").unwrap()
            ==
            ColumnExp {
                name: None,
                ty: Some("string".into()),
                es: vec!(
                    Exp::Scalar(Scalar::UTF8("hello".into())),
                    Exp::Scalar(Scalar::UTF8("world".into())),
                    )
            });

    assert!(StatementParser::new().parse("let x = [a:i; 1 2 3];").unwrap()
            ==
            Stmt::Let(LetKind::Imm, "x".into(), Exp::Column(ColumnExp {
                name: Some("a".into()),
                ty: Some("i".into()),
                es: vec!(
                    Exp::Scalar(Scalar::I32(1)),
                    Exp::Scalar(Scalar::I32(2)),
                    Exp::Scalar(Scalar::I32(3)))
            }).into()));

    assert!(RangeLiteralParser::new().parse("(1..llama_world)").unwrap()
            == RangeExp {
                start: Exp::Scalar(Scalar::I32(1)).into(),
                end: Exp::Name("llama_world".into()).into(),
            });

    assert!(StatementParser::new().parse("if true then 3 else 4 end").unwrap()
            ==
            Stmt::IfElse(
                Exp::Name("true".into()).into(),
                Exp::Scalar(Scalar::I32(3)).into(),
                Exp::Scalar(Scalar::I32(4)).into()));

    assert!(ExprParser::new().parse("{true; 1; 3}").unwrap()
            ==
            Exp::Block(
                vec![
                Stmt::Exp(Exp::Name("true".into()).into()),
                Stmt::Exp(Exp::Scalar(Scalar::I32(1)).into()),
                ],
                Exp::Scalar(Scalar::I32(3)).into()
                ));
}
