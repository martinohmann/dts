/// Represents a HCL number.
#[derive(Debug, PartialEq)]
pub enum Number {
    /// Represents a integer.
    Int(i64),
    /// Represents a float.
    Float(f64),
}
