use std::fmt;

/// The body of a HCL config file.
pub type Body<'a> = Vec<Structure<'a>>;

/// Possible HCL Structures.
#[derive(Debug, PartialEq)]
pub enum Structure<'a> {
    /// An Attribute is a key-value pair where the key is a string identifier. The value can be a
    /// literal value or complex expression.
    Attribute(&'a str, Expression<'a>),
    /// A nested block which has an identifier and zero or more keys.
    Block(Vec<&'a str>, Box<Body<'a>>),
}

#[derive(Debug, PartialEq)]
pub enum Expression<'a> {
    Value(Value<'a>),
    TemplateExpr(&'a str),
    /// Any other expression.
    RawExpr(&'a str),
    /// Raw operation expression.
    Operation(&'a str),
    /// Raw conditional expression.
    Conditional(&'a str),
}

impl fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Value(val) => write!(f, "{}", val),
            Expression::TemplateExpr(tpl) => write!(f, "{}", tpl),
            Expression::RawExpr(raw) => write!(f, "{}", raw),
            Expression::Operation(op) => write!(f, "{}", op),
            Expression::Conditional(cond) => write!(f, "{}", cond),
        }
    }
}

impl<'a> Expression<'a> {
    /// Interpolate the expression as a string by wrapping it into `${` and `}` if it is neither a
    /// literal nor a collection value.
    pub fn interpolate(&self) -> String {
        let raw = match self {
            Expression::RawExpr(raw) => raw,
            Expression::Operation(op) => op,
            Expression::Conditional(cond) => cond,
            _ => return self.to_string(),
        };

        format!("${{{}}}", raw)
    }
}

/// Represents any valid HCL value.
#[derive(Debug, PartialEq)]
pub enum Value<'a> {
    /// Represents a HCL null value.
    Null,
    /// Represents a HCL boolean.
    Bool(bool),
    /// Represents a HCL number, either integer or float.
    Number(Number),
    /// Represents a HCL string.
    String(&'a str),
    /// Represents a HCL tuple.
    Tuple(Vec<Expression<'a>>),
    /// Represents a HCL object.
    Object(Vec<ObjectItem<'a>>),
}

impl fmt::Display for Value<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(num) => write!(f, "{}", num),
            Value::String(s) => write!(f, "{}", s),
            Value::Tuple(tuple) => {
                let items = tuple
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "[{}]", items)
            }
            Value::Object(object) => {
                let items = object
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "{{{}}}", items)
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct ObjectItem<'a>(pub ObjectKey<'a>, pub Expression<'a>);

impl fmt::Display for ObjectItem<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.0, self.1)
    }
}

#[derive(Debug, PartialEq)]
pub enum ObjectKey<'a> {
    Identifier(&'a str),
    Expression(Expression<'a>),
}

impl fmt::Display for ObjectKey<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectKey::Identifier(ident) => write!(f, "{}", ident),
            ObjectKey::Expression(expr) => write!(f, "{}", expr),
        }
    }
}

/// Represents a HCL number.
#[derive(Debug, PartialEq)]
pub enum Number {
    /// Represents a integer.
    Int(i64),
    /// Represents a float.
    Float(f64),
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Int(int) => write!(f, "{}", int),
            Number::Float(float) => write!(f, "{}", float),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn interpolate() {
        let cond = Expression::Conditional("var.enabled ? 1 : 0");
        assert_eq!(&cond.interpolate(), "${var.enabled ? 1 : 0}");

        let op = Expression::Operation("!var.enabled");
        assert_eq!(&op.interpolate(), "${!var.enabled}");

        let raw = Expression::RawExpr("toset(var.foo)");
        assert_eq!(&raw.interpolate(), "${toset(var.foo)}");

        let boolean = Expression::Value(Value::Bool(true));
        assert_eq!(&boolean.interpolate(), "true");

        let string = Expression::Value(Value::String("foobar"));
        assert_eq!(&string.interpolate(), "foobar");
    }
}
