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
    ExprTerm(ExprTerm<'a>),
    Operation(&'a str),
    Conditional(&'a str),
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

/// Represents any valid HCL value.
#[derive(Debug, PartialEq)]
pub enum CollectionValue<'a> {
    /// Represents a HCL tuple.
    Tuple(Vec<Expression<'a>>),
    /// Represents a HCL object.
    Object(Vec<ObjectItem<'a>>),
}

#[derive(Debug, PartialEq)]
pub struct ObjectItem<'a>(pub ObjectKey<'a>, pub Expression<'a>);

#[derive(Debug, PartialEq)]
pub enum ObjectKey<'a> {
    Identifier(&'a str),
    Expression(Expression<'a>),
}

/// Represents a HCL number.
#[derive(Debug, PartialEq)]
pub enum Number {
    /// Represents a integer.
    Int(i64),
    /// Represents a float.
    Float(f64),
}
