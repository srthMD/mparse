use crate::{eval::EvaluationErrorRepr, types::object::Object};

pub(crate) trait FieldAccess {
  fn access_field(&self, field_name: String) -> Result<Object, EvaluationErrorRepr>;
}
