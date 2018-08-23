#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use] extern crate lazy_static;
#[macro_use] extern crate lalrpop_util;

// Keep until lalrpop 0.16
lalrpop_mod!(pub tablam);
// pub mod tablam;

mod core;
mod tok;

use std::io::{self, Write, BufRead};
// use core::operators::*;
use tok::{TablamTokenizer};


fn t<'input>(s: &'input str) -> TablamTokenizer<'input> {
    TablamTokenizer::new(s)
}

fn main() -> Result<(), Box<std::error::Error>> {
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

    use std::env;
    use core::ast::*;
    use std::fs::File;
    use std::io::Read;
    use tablam::ProgramParser;
    // use core::typecheck::*;


    let args: Vec<String> = env::args().collect();
    let filename: &String = args.get(1).ok_or("Did not pass a filename on CLI")?;
    let mut f = File::open(filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;
    let ast = ProgramParser::new().parse(t(&contents))?;
    let res = ast.run(vec!(), &Env::empty())?;
    println!("returned: {:?}", res);

    // let e = TypedExp {
    //     ty: Ty::Atype(Atype::Conid("Int".into()).into()),
    //     exp: Exp::Scalar(Scalar::I32(1))
    // };
    // let e = Kind::Arrow(vec!(Kind::Type, Kind::Type), Kind::Type.into());
    // let e = listtype(I32TYPE);
    // println!("e: {}", e);


    // loop {
    //     use tablam::*;

    //     let stdin = io::stdin();
    //     print!("> ");
    //     std::io::stdout().flush().unwrap();
    //     for line_r in stdin.lock().lines() {
    //         let line = line_r.unwrap();

    //         let typarser = TypeParser::new();
    //         let tlparser = ProgBlockParser::new();
    //         let parser = StatementParser::new();
    //         let eparser = ExprParser::new();

    //         println!("{:?}", parser.parse(t(&line)));

    //         // match tlparser.parse(t(&line)) {
    //         //     Ok(ast) => println!("ok(P): {:?}", ast),
    //         //     Err(err) => match parser.parse(t(&line)) {
    //         //         Ok(ast) => println!("ok(S): {:?}", ast),
    //         //         Err(err) => match eparser.parse(t(&line)) {
    //         //             Ok(ast) => println!("ok(E): {:?}", ast),
    //         //             Err(err) => println!("error: {:?}", err),
    //         //         }
    //         //     }
    //         // }

    //         print!("> ");
    //         std::io::stdout().flush().unwrap();
    //     }
    // }
    Ok(())
}

// use core::ast::Exp;
// fn name(s: &str) -> Exp {
//     Exp::Name(s.into())
// }
// 
// fn stringlit(s: &str) -> Exp {
//     use core::types::{Scalar, encode_str};
//     Exp::Scalar(Scalar::UTF8(encode_str(s)))
// }
// 
// use std::rc::Rc;
// fn rc<T>(s: T) -> Rc<T> {
//     Rc::new(s)
// }
// 
// use core::ast::{Ty, Atype};
// fn star(s: &str) -> Ty {
//     Ty::Arrow(Ty::Atype(Atype::Conid(s.into()).into()).into(), None.into())
// }

#[test]
fn tablam() {
    use tablam::*;
    // It is unclear why I have to import both ColumnExp and * ... ???
    use core::ast::{ColumnExp, *};

    let contents = include_str!("example.tb");
    match ProgramParser::new().parse(t(&contents)) {
        Ok(ast) => (),
        Err(err) => panic!(format!("{:?}", err)),
    };

    assert!(
        ExprParser::new().parse(t("12")).unwrap()
        == 12i32.into());

    assert!(
        ExprParser::new().parse(t("12i64")).unwrap()
        == 12i64.into());

    assert!(
        ExprParser::new().parse(t("LLAMA")).unwrap()
        == Exp::Constant("LLAMA".into())
        );

    assert!(
        ExprParser::new().parse(t("true")).unwrap()
        == true.into()
        );

    assert!(
        ExprParser::new().parse(t("false")).unwrap()
        == false.into()
        );

    assert!(
        ExprParser::new().parse(t(r#""hello""#)).unwrap()
        == stringlit("hello")
        );

    // assert!(ExprParser::new().parse(t(r#""hello"#)).is_err());

    assert!(ExprParser::new().parse(t("1")).unwrap()
            == 1i32.into());

    assert!(ExprParser::new().parse(t("1 + 2")).unwrap()
            == Exp::BinOp(BinOp::Plus, rc(1.into()), rc(2.into())));

    assert!(ExprParser::new().parse(t("1 + /* hello world */ 2")).unwrap()
            == Exp::BinOp(BinOp::Plus, rc(1.into()), rc(2.into())));

    assert!(StatementParser::new().parse(t("{1; 2; 3;}")).unwrap()
            ==
            Stmt::Block(vec!(
                    Stmt::Exp(1.into()),
                    Stmt::Exp(2.into()),
                    Stmt::Exp(3.into()),
                    ))
            );

    assert!(ExprParser::new().parse(t("do 1; 2; 3 end")).unwrap()
            ==
            Exp::Block(
                vec!(Stmt::Exp(1.into()), Stmt::Exp(2.into())),
                rc(3.into())
                )
            );

    assert!(ColumnLiteralParser::new().parse(t("[name:String; \"hello\" \"world\"]")).unwrap()
            ==
            ColumnExp {
                name: Some("name".into()),
                ty: Some(star("String")),
                es: vec!(stringlit("hello"), stringlit("world")),
            });

    assert!(ColumnLiteralParser::new().parse(t("[String; \"hello\" \"world\"]")).unwrap()
            ==
            ColumnExp {
                name: None,
                ty: Some(star("String")),
                es: vec!(stringlit("hello"), stringlit("world")),
            });

    assert!(ExprParser::new().parse(t("List[1 2]")).unwrap()
            ==
            Exp::Container(
                star("List"),
                ColumnExp { name: None, ty: None, es: vec!(1.into(), 2.into()) }
                ));

    assert!(ExprParser::new().parse(t("a.b.c.d")).unwrap()
            ==
                Exp::ColumnSelect(
                    Exp::ColumnSelect(
                        Exp::ColumnSelect(
                            name("a").into(),
                            "b".into()
                        ).into(),
                        "c".into()
                    ).into(),
                "d".into()));

    assert!(ExprParser::new().parse(t("a.0.1.2")).unwrap()
            ==
                Exp::ColumnSelect(
                    Exp::ColumnSelect(
                        Exp::ColumnSelect(
                            name("a").into(),
                            0.into()
                        ).into(),
                        1.into()
                    ).into(),
                2.into()));

    assert!(StatementParser::new().parse(t("let x = [a:Ty; 1 2 3];")).unwrap()
            ==
            Stmt::Let(LetKind::Imm, "x".into(), None, Exp::Column(ColumnExp {
                name: Some("a".into()),
                ty: Some(star("Ty")),
                es: vec![1i32.into(), 2i32.into(), 3i32.into()],
            }).into()));

    assert!(RelationLiteralParser::new().parse(t(r#"[<id, name; 1 "1"; 2 "2" >]"#)).unwrap()
            ==
            RelationExp {
                rel_type: RelType::Row,
                names: vec!("id".into(), "name".into()),
                data: vec!(
                    vec!(1.into(), stringlit("1")),
                    vec!(2.into(), stringlit("2")),
                    )
            });

    assert!(RelationLiteralParser::new().parse(t(r#"[| id= 1 2; name= "1" "2" |]"#)).unwrap()
            ==
            RelationExp {
                rel_type: RelType::Col,
                names: vec!("id".into(), "name".into()),
                data: vec!(
                    vec!(1.into(), 2.into()),
                    vec!(stringlit("1"), stringlit("2")),
                    )
            });

    assert!(RangeLiteralParser::new().parse(t("(1..llama_world)")).unwrap()
            == RangeExp {
                start: rc(1i32.into()),
                end: name("llama_world").into(),
            });

    assert!(StatementParser::new().parse(t("if true 3 else 4")).unwrap()
            ==
            Stmt::IfElse(
                rc(true.into()),
                rc(3i32.into()),
                rc(4i32.into())));

    assert!(RowLiteralParser::new().parse(t("{hello=1, world=true}")).unwrap()
            ==
            RowExp {
                names: Some(vec!["hello".into(), "world".into()]),
                types: vec!(None, None),
                es: vec![1i32.into(), true.into()],
            });

    assert!(RowLiteralParser::new().parse(t("{hello:Int=1, world=true}")).unwrap()
            ==
            RowExp {
                names: Some(vec!["hello".into(), "world".into()]),
                types: vec!(Some(star("Int")), None),
                es: vec![1i32.into(), true.into()],
            });

    assert!(RowLiteralParser::new().parse(t("{1, true}")).unwrap()
            ==
            RowExp {
                names: None,
                types: vec!(None, None),
                es: vec![1i32.into(), true.into()],
            });

    assert!(StatementParser::new().parse(t("while true do hello; end")).unwrap()
            ==
            Stmt::While(
                rc(true.into()),
                Stmt::Block(
                    vec![Stmt::Exp(name("hello"))],
                ).into()));

    assert!(ColumnLiteralParser::new().parse(t("[1 2 3 (4+5)]")).unwrap()
            ==
            ColumnExp {
                name: None,
                ty: None,
                es: vec![
                    1i32.into(),
                    2i32.into(),
                    3i32.into(),
                    Exp::BinOp(
                        BinOp::Plus,
                        rc(4i32.into()),
                        rc(5i32.into())
                    ),
                ]
            });

    assert!(ExprParser::new().parse(t("()")).unwrap() == Exp::Unit);

    assert!(TypeParser::new().parse(t("Int")).unwrap()
            == star("Int"));

    assert!(TypeParser::new().parse(t("Int -> String -> Float")).unwrap()
            == Ty::Arrow(
                    Ty::Atype(Atype::Conid(("Int").into()).into()).into(),
                    Some(Ty::Arrow(
                            Ty::Atype(Atype::Conid(("String").into()).into()).into(),
                            Some(star("Float"),
                            ).into())).into()));

    assert!(ExprParser::new().parse(t("a ? # name == \"Max\" # your != mom")).unwrap()
            == Exp::QueryFilter(
                Exp::Name("a".into()).into(),
                vec!(
                    FilterExp::RelOp(
                        RelOp::Equals,
                        "name".into(),
                        stringlit("Max").into()),
                    FilterExp::RelOp(
                        RelOp::NotEquals,
                        "your".into(),
                        name("mom").into()),
                    )));

    assert!(ExprParser::new().parse(t("[1 2 3] # 1")).unwrap()
            ==
            Exp::QuerySelect(
                Exp::Column(ColumnExp { name: None, ty: None, es: vec!(1.into(), 2.into(), 3.into()) }).into(),
                rc(1.into())
                ));

    assert!(ExprParser::new().parse(t("[1 2 3] # [1 2]")).unwrap()
            ==
            Exp::QuerySelect(
                Exp::Column(ColumnExp { name: None, ty: None, es: vec!(1.into(), 2.into(), 3.into()) }).into(),
                Exp::Column(ColumnExp { name: None, ty: None, es: vec!(1.into(), 2.into()) }).into(),
                ));

    assert!(ExprParser::new().parse(t("[1 2 3] # (1..2)")).unwrap()
            ==
            Exp::QuerySelect(
                Exp::Column(ColumnExp { name: None, ty: None, es: vec!(1.into(), 2.into(), 3.into()) }).into(),
                Exp::Range(RangeExp { start: rc(1.into()), end: rc(2.into()) }).into(),
                ));

    assert!(ProgBlockParser::new().parse(t("fun test(a:Int, b:String): Int = do 1 + 2 end")).unwrap()
            ==
            ProgBlock::Function(FunDef {
                name: "test".into(),
                params: vec!(
                    ("a".into(), star("Int")),
                    ("b".into(), star("String"))
                    ),
                ret_ty: star("Int"),
                body: Exp::Block(
                    vec!(),
                    rc(Exp::BinOp(
                            BinOp::Plus,
                            rc(1.into()),
                            rc(2.into()))))
            }));

    assert!(ExprParser::new().parse(t("print(a(b), c(d))")).unwrap()
            ==
            Exp::Apply(
                name("print").into(),
                vec!(
                    Exp::Apply(name("a").into(), vec!(name("b"))),
                    Exp::Apply(name("c").into(), vec!(name("d"))),
                    )));

    assert!(ExprParser::new().parse(t("a | b | c")).unwrap()
            ==
            Exp::Apply(
                name("c").into(),
                vec!(Exp::Apply(
                        name("b").into(),
                        vec!(name("a").into())))));

    assert!(ExprParser::new().parse(t("1 + 2 | print")).unwrap()
            ==
            Exp::Apply(
                name("print").into(),
                vec!(Exp::BinOp(BinOp::Plus, rc(1.into()), rc(2.into())))));

    assert!(StatementParser::new().parse(t(r#"
                for row in city ? #name == "new york" do
                    row | print;
                end"#)).unwrap()
            ==
            Stmt::For(
                "row".into(),
                Exp::QueryFilter(
                    name("city").into(),
                    vec!(FilterExp::RelOp(
                            RelOp::Equals,
                            "name".into(),
                            stringlit("new york").into()))
                ).into(),
                Stmt::Block(
                    vec!(
                        Stmt::Exp(Exp::Apply(name("print").into(), vec!(name("row").into())))
                    )
                ).into()));

    assert!(ProgBlockParser::new().parse(t("HELLO:Int = 123")).unwrap()
            ==
            ProgBlock::Constant("HELLO".into(), star("Int"), rc(123.into())));
}
