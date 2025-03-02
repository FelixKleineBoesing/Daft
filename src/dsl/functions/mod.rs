pub mod float;
pub mod image;
pub mod list;
pub mod numeric;
pub mod temporal;
pub mod utf8;

use self::float::FloatExpr;
use self::image::ImageExpr;
use self::list::ListExpr;
use self::numeric::NumericExpr;
use self::temporal::TemporalExpr;
use self::utf8::Utf8Expr;
use crate::{datatypes::Field, error::DaftResult, schema::Schema, series::Series};
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
pub mod python;
#[cfg(feature = "python")]
use python::PythonUDF;

use super::Expr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FunctionExpr {
    Numeric(NumericExpr),
    Float(FloatExpr),
    Utf8(Utf8Expr),
    Temporal(TemporalExpr),
    List(ListExpr),
    Image(ImageExpr),
    #[cfg(feature = "python")]
    Python(PythonUDF),
}

pub trait FunctionEvaluator {
    fn fn_name(&self) -> &'static str;
    fn to_field(&self, inputs: &[Expr], schema: &Schema) -> DaftResult<Field>;
    fn evaluate(&self, inputs: &[Series], expr: &Expr) -> DaftResult<Series>;
}

impl FunctionExpr {
    #[inline]
    fn get_evaluator(&self) -> &dyn FunctionEvaluator {
        use FunctionExpr::*;
        match self {
            Numeric(expr) => expr.get_evaluator(),
            Float(expr) => expr.get_evaluator(),
            Utf8(expr) => expr.get_evaluator(),
            Temporal(expr) => expr.get_evaluator(),
            List(expr) => expr.get_evaluator(),
            Image(expr) => expr.get_evaluator(),
            #[cfg(feature = "python")]
            Python(expr) => expr,
        }
    }
}

impl FunctionEvaluator for FunctionExpr {
    fn fn_name(&self) -> &'static str {
        self.get_evaluator().fn_name()
    }

    fn to_field(&self, inputs: &[Expr], schema: &Schema) -> DaftResult<Field> {
        self.get_evaluator().to_field(inputs, schema)
    }

    fn evaluate(&self, inputs: &[Series], expr: &Expr) -> DaftResult<Series> {
        self.get_evaluator().evaluate(inputs, expr)
    }
}
