//! Data transformation utilities.

use crate::{Error, Result, Value};
use jsonpath_rust::JsonPathQuery;

/// Filter value in place according to the jsonpath query.
///
/// This can fail if parsing the query fails.
pub fn filter_in_place<Q>(value: &mut Value, query: Q) -> Result<()>
where
    Q: AsRef<str>,
{
    value
        .clone()
        .path(query.as_ref())
        .map(|filtered| *value = filtered)
        .map_err(Error::new)
}

/// Remove one level of nesting if the data is shaped like an array.
///
/// If the has only one element the array will be removed entirely, leaving the single element as
/// output.
pub fn flatten_in_place(value: &mut Value) {
    if let Some(array) = value.as_array() {
        *value = if array.len() == 1 {
            array[0].clone()
        } else {
            Value::Array(
                array
                    .iter()
                    .map(|v| match v {
                        Value::Array(a) => a.clone(),
                        _ => vec![v.clone()],
                    })
                    .flatten()
                    .collect(),
            )
        };
    }
}
