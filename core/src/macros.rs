use super::types::*;
use rust_decimal::Decimal;

macro_rules! convert {
    ($kind:ident, $bound:path) => (
        impl <'a> From<&'a $kind> for Scalar {
            fn from(i: &'a $kind) -> Self {
                $bound(i.clone())
            }
        }

        impl From<$kind> for Scalar {
            fn from(i: $kind) -> Self {
                $bound(i)
            }
        }

        impl From<Option<$kind>> for Scalar {
            fn from(i: Option<$kind>) -> Self {
                match i {
                    Some(x) => $bound(x),
                    _ => Scalar::None,
                }                
            }
        }

        impl From<Scalar> for $kind {
            fn from(i: Scalar) -> Self {
                match i {
                    $bound(x) => x,
                    _ => unreachable!("{:?}", i)
                }
            }
        }

        impl <'a> From<&'a Scalar> for $kind {
            fn from(i: &'a Scalar) -> Self {
                match i {
                    $bound(x) => x.clone(),
                    _ => unreachable!("{:?}", i)
                }
            }
        }

        impl <'a> From<&'a Scalar> for Option<$kind> {
            fn from(i: &'a Scalar) -> Self {
                match i {
                    $bound(x) => Some(x.clone()),
                    _ => None
                }
            }
        }
    )
}

convert!(bool, Scalar::Bool);
convert!(isize, Scalar::ISize);
convert!(i32, Scalar::I32);
convert!(i64, Scalar::I64);
convert!(TimeStamp, Scalar::DateTime);
convert!(Decimal, Scalar::Decimal);
convert!(String, Scalar::UTF8);

macro_rules! convert_rel {
    ($source:ident, $dest:ident) => (

        impl <'a> From<&'a $source> for $dest {
            fn from(source: &'a $source) -> Self {
                $dest::new_from(source.schema.clone(), source)
            }
        }

    )
}

convert_rel!(Table, BTree);
convert_rel!(BTree, Table);