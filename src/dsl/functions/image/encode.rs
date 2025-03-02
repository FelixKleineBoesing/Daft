use crate::{
    datatypes::{DataType, Field},
    dsl::{functions::FunctionExpr, Expr},
    error::{DaftError, DaftResult},
    schema::Schema,
    series::Series,
};

use super::{super::FunctionEvaluator, ImageExpr};

pub struct EncodeEvaluator {}

impl FunctionEvaluator for EncodeEvaluator {
    fn fn_name(&self) -> &'static str {
        "encode"
    }

    fn to_field(&self, inputs: &[Expr], schema: &Schema) -> DaftResult<Field> {
        match inputs {
            [input] => {
                let field = input.to_field(schema)?;
                match field.dtype {
                    DataType::Image(..) => Ok(Field::new(field.name, DataType::Binary)),
                    _ => Err(DaftError::TypeError(format!(
                        "ImageEncode can only encode ImageArrays, got {}",
                        field
                    ))),
                }
            }
            _ => Err(DaftError::SchemaMismatch(format!(
                "Expected 1 input arg, got {}",
                inputs.len()
            ))),
        }
    }

    fn evaluate(&self, inputs: &[Series], expr: &Expr) -> DaftResult<Series> {
        let image_format = match expr {
            Expr::Function {
                func: FunctionExpr::Image(ImageExpr::Encode { image_format }),
                inputs: _,
            } => image_format,
            _ => panic!("Expected ImageEncode Expr, got {expr}"),
        };
        match inputs {
            [input] => input.image_encode(*image_format),
            _ => Err(DaftError::ValueError(format!(
                "Expected 1 input arg, got {}",
                inputs.len()
            ))),
        }
    }
}
