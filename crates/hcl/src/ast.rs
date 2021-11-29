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
    /// An expression term like literal value or template expression.
    ExprTerm(ExprTerm<'a>),
    /// Raw operation expression.
    Operation(&'a str),
    /// Raw conditional expression.
    Conditional(&'a str),
}

impl fmt::Display for Expression<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::ExprTerm(term) => write!(f, "{}", term),
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
            Expression::ExprTerm(ExprTerm::RawExpr(raw)) => raw,
            Expression::Operation(op) => op,
            Expression::Conditional(cond) => cond,
            _ => return self.to_string(),
        };

        format!("${{{}}}", raw)
    }
}

/// Represents a HCL expression.
#[derive(Debug, PartialEq)]
pub enum ExprTerm<'a> {
    LiteralValue(LiteralValue<'a>),
    CollectionValue(CollectionValue<'a>),
    TemplateExpr(&'a str),
    /// Any other expression.
    RawExpr(&'a str),
}

impl fmt::Display for ExprTerm<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprTerm::LiteralValue(val) => write!(f, "{}", val),
            ExprTerm::CollectionValue(val) => write!(f, "{}", val),
            ExprTerm::TemplateExpr(tpl) => write!(f, "{}", tpl),
            ExprTerm::RawExpr(raw) => write!(f, "{}", raw),
        }
    }
}

/// Represents any valid HCL value.
#[derive(Debug, PartialEq)]
pub enum LiteralValue<'a> {
    /// Represents a HCL null value.
    Null,
    /// Represents a HCL boolean.
    Bool(bool),
    /// Represents a HCL number, either integer or float.
    Number(Number),
    /// Represents a HCL string.
    String(&'a str),
}

impl fmt::Display for LiteralValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LiteralValue::Null => write!(f, "null"),
            LiteralValue::Bool(b) => write!(f, "{}", b),
            LiteralValue::Number(num) => write!(f, "{}", num),
            LiteralValue::String(s) => write!(f, "{}", s),
        }
    }
}

/// Represents any valid HCL value.
#[derive(Debug, PartialEq)]
pub enum CollectionValue<'a> {
    /// Represents a HCL tuple.
    Tuple(Vec<Expression<'a>>),
    /// Represents a HCL object.
    Object(Vec<ObjectItem<'a>>),
}

impl fmt::Display for CollectionValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollectionValue::Tuple(tuple) => {
                let items = tuple
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(f, "[{}]", items)
            }
            CollectionValue::Object(object) => {
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

        let raw = Expression::ExprTerm(ExprTerm::RawExpr("toset(var.foo)"));
        assert_eq!(&raw.interpolate(), "${toset(var.foo)}");

        let boolean = Expression::ExprTerm(ExprTerm::LiteralValue(LiteralValue::Bool(true)));
        assert_eq!(&boolean.interpolate(), "true");

        let string = Expression::ExprTerm(ExprTerm::LiteralValue(LiteralValue::String("foobar")));
        assert_eq!(&string.interpolate(), "foobar");
    }
}
