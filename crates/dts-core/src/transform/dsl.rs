//! Provides a DSL to define transformations and parse user input.

use crate::parsers::func_sig::{self, ExprTerm, FuncArg, FuncSig};
use crate::{Error, Result};
use dts_json::{Number, Value};
use indexmap::{IndexMap, IndexSet};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{self, Write};
use std::str::FromStr;

/// Represents the definition of a transformation.
///
/// ## Example
///
/// ```
/// use dts_core::transform::dsl::{Arg, Definition};
///
/// let definition = Definition::new("sort")
///     .with_description("Sorts the input based on the value of the `order` argument")
///     .add_alias("s")
///     .add_arg(Arg::new("order").with_default_value("asc"));
/// ```
#[derive(Default, Clone)]
pub struct Definition<'a> {
    name: &'a str,
    aliases: IndexSet<&'a str>,
    description: Option<&'a str>,
    args: IndexMap<&'a str, Arg<'a>>,
}

impl<'a> Definition<'a> {
    /// Creates a new transformation `Definition` with the given name.
    pub fn new(name: &'a str) -> Self {
        Definition {
            name,
            ..Default::default()
        }
    }

    /// Returns the name of the transformation.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the description of the transformation or `None`.
    pub fn description(&self) -> Option<&'a str> {
        self.description
    }

    /// Returns a reference to the aliases for this `Definition`.
    pub fn aliases(&self) -> &IndexSet<&'a str> {
        &self.aliases
    }

    /// Returns a reference to the arguments for this `Definition`.
    pub fn args(&self) -> &IndexMap<&'a str, Arg<'a>> {
        &self.args
    }

    /// Adds an alias to the `Definition` and returns it.
    pub fn add_alias(mut self, alias: &'a str) -> Self {
        self.aliases.insert(alias);
        self
    }

    /// Adds multiple aliases to the `Definition` and returns it.
    pub fn add_aliases(self, aliases: &[&'a str]) -> Self {
        aliases
            .iter()
            .fold(self, |definition, alias| definition.add_alias(alias))
    }

    /// Sets the description for the `Definition` and returns it.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Adds an argument to the `Definition` and returns it.
    pub fn add_arg<T>(mut self, arg: T) -> Self
    where
        T: Into<Arg<'a>>,
    {
        let arg = arg.into();
        self.args.insert(arg.name, arg);

        // Argument order:
        // 1. required
        // 2. optional without default value
        // 3. optional with default value
        self.args
            .sort_by(|_, a, _, b| match b.is_required().cmp(&a.is_required()) {
                Ordering::Equal => match (a.default_value(), b.default_value()) {
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (_, _) => a.is_required().cmp(&b.is_required()),
                },
                non_eq => non_eq,
            });
        self
    }

    /// Adds multiple arguments to the `Definition` and returns it.
    pub fn add_args<I, T>(self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Arg<'a>>,
    {
        args.into_iter()
            .fold(self, |definition, arg| definition.add_arg(arg))
    }

    /// Returns `true` if any of the definition's attributes (e.g. name, description, aliases or
    /// args) contains the keyword. The keyword is assumed to be lowercase.
    ///
    /// This is useful for search.
    pub fn contains_keyword(&self, keyword: &str) -> bool {
        self.name().to_lowercase().contains(keyword)
            || self
                .description()
                .map(|desc| desc.to_lowercase())
                .map(|desc| desc.contains(keyword))
                .unwrap_or(false)
            || self
                .aliases()
                .iter()
                .map(|alias| alias.to_lowercase())
                .any(|alias| alias.contains(keyword))
            || self
                .args()
                .values()
                .any(|arg| arg.contains_keyword(keyword))
    }

    // Matches a list of function arguments against the `Definition` and returns a `HashMap` of
    // argument name to argument value or an error if a required argument is missing. Optional
    // arguments receive their default value if they did not appear in `func_args`.
    fn match_func_args(
        &self,
        definitions: &Definitions<'a>,
        func_args: &[FuncArg<'a>],
    ) -> Result<HashMap<&'a str, ArgMatch<'a>>> {
        let mut remaining_args = self.args.clone();
        let mut args: HashMap<&'a str, ArgMatch<'a>> = HashMap::new();

        for arg in func_args.iter() {
            let (name, expr_term) = match arg {
                FuncArg::Named(name, expr_term) => remaining_args
                    .shift_remove(name)
                    .map(|arg_def| (arg_def.name, expr_term))
                    .ok_or_else(|| {
                        Error::new(format!(
                            "Unexpected named argument `{}={}`",
                            name, expr_term
                        ))
                    })?,
                FuncArg::Positional(expr_term) => remaining_args
                    .shift_remove_index(0)
                    .map(|(_, arg_def)| (arg_def.name, expr_term))
                    .ok_or_else(|| {
                        Error::new(format!("Unexpected positional argument `{}`", expr_term))
                    })?,
            };

            if args.contains_key(name) {
                return Err(Error::new(format!("Duplicate argument `{}`", name)));
            }

            let arg_match = match expr_term {
                ExprTerm::Value(value) => {
                    ArgMatch::Value(serde_json::from_str(value).map_err(|err| {
                        Error::new(format!("Invalid value for argument `{}`: {}", name, err))
                    })?)
                }
                ExprTerm::Expr(func_sigs) => ArgMatch::Expr(
                    func_sigs
                        .iter()
                        .map(|func_sig| definitions.match_definition(func_sig))
                        .collect::<Result<Vec<_>>>()?,
                ),
            };

            args.insert(name, arg_match);
        }

        let mut missing_args = Vec::new();

        for (name, arg_def) in remaining_args.into_iter() {
            match arg_def.default_value() {
                Some(default_value) => {
                    args.insert(name, ArgMatch::Value(default_value.clone()));
                }
                None => {
                    if arg_def.is_required() {
                        missing_args.push(name);
                    }
                }
            }
        }

        if !missing_args.is_empty() {
            return Err(Error::new(format!(
                "Required arguments missing: {}",
                missing_args.join(","),
            )));
        }

        Ok(args)
    }
}

impl<'a> fmt::Display for Definition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name)?;
        f.write_char('(')?;

        let mut optional_args = 0;

        for (i, arg) in self.args.values().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }

            if !arg.is_required() {
                f.write_char('[')?;
                optional_args += 1;
            }

            f.write_str(&arg.to_string())?;
        }

        // close the brackets for all optional args.
        for _ in 0..optional_args {
            f.write_char(']')?;
        }

        f.write_char(')')
    }
}

/// A type that acts as a registry for available transformation definitions.
///
/// Its main functionality is exposed via `add_definition` for adding a `Definition` to the
/// registry and `parse` for parsing user input into a `Vec<DefinitionMatch>`.
///
/// The `Definitions` registry can also be used as the starting point to dynamically generate
/// documentation for all available transformations.
///
/// ## Example
///
/// ```
/// use dts_core::transform::dsl::{Arg, Definition, Definitions};
///
/// let definitions = Definitions::new()
///     .add_definition(
///         Definition::new("sort")
///             .add_arg(Arg::new("order"))
///     );
/// ```
#[derive(Default, Clone)]
pub struct Definitions<'a> {
    inner: Vec<Definition<'a>>,
}

impl<'a> Definitions<'a> {
    /// Creates a new `Definitions` registry.
    pub fn new() -> Self {
        Default::default()
    }

    /// Consumes `self` and returns the inner `Vec<Definition>`. This can be used to iterate all
    /// available definitions.
    pub fn into_inner(self) -> Vec<Definition<'a>> {
        self.inner
    }

    /// Adds a `Definition` to the registry and returns it.
    pub fn add_definition(mut self, definition: Definition<'a>) -> Self {
        self.inner.push(definition);
        self
    }

    // Finds a definition in the registry. Aliases are used for the lookup as well.
    //
    // Returns `None` if no definition matching the provided name or alias exists.
    fn find_definition(&self, name: &'a str) -> Option<&Definition<'a>> {
        self.inner.iter().find(|definition| {
            definition.name == name || definition.aliases.iter().any(|&alias| alias == name)
        })
    }

    /// Parses transformations from a `&str` and returns a `Vec<DefinitionMatch>` which contains
    /// the names and resolved arguments of the matching definitions.
    ///
    /// ## Example
    ///
    /// ```
    /// use dts_core::transform::dsl::{Arg, Definition, Definitions};
    ///
    /// # use pretty_assertions::assert_eq;
    /// # use std::error::Error;
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let definitions = Definitions::new()
    ///     .add_definition(
    ///         Definition::new("sort")
    ///             .add_arg(Arg::new("order"))
    ///     );
    ///
    /// let matches = definitions.parse("sort(\"asc\")")?;
    ///
    /// for m in matches {
    ///     match m.name() {
    ///         "sort" => {
    ///             let order = m.str_value("order")?;
    ///             assert_eq!(order, "asc");
    ///         }
    ///         _ => unimplemented!()
    ///     }
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn parse(&self, expression: &'a str) -> Result<Vec<DefinitionMatch<'a>>> {
        func_sig::parse(expression)?
            .iter()
            .map(|func_sig| self.match_definition(func_sig))
            .collect()
    }

    fn match_definition(&self, func_sig: &FuncSig<'a>) -> Result<DefinitionMatch<'a>> {
        let definition = self
            .find_definition(func_sig.name())
            .ok_or_else(|| Error::new(format!("Unknown function `{}`", func_sig.name())))?;

        let args = definition
            .match_func_args(self, func_sig.args())
            .map_err(|err| {
                Error::new(format!(
                    "Invalid function signature `{}`: {}",
                    func_sig, err
                ))
            })?;

        Ok(DefinitionMatch::new(definition.name, args))
    }
}

/// Represents an argument for a transformation.
///
/// ## Example
///
/// ```
/// use dts_core::transform::dsl::Arg;
///
/// let arg = Arg::new("order")
///     .with_description("The sorting order")
///     .with_default_value("asc");
/// ```
#[derive(Default, Clone)]
pub struct Arg<'a> {
    name: &'a str,
    required: bool,
    default_value: Option<Value>,
    description: Option<&'a str>,
}

impl<'a> Arg<'a> {
    /// Create a new named `Arg`.
    pub fn new(name: &'a str) -> Self {
        Arg {
            name,
            required: true,
            ..Default::default()
        }
    }

    /// Returns the argument's name.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns `true` if the argument is required.
    pub fn is_required(&self) -> bool {
        self.required
    }

    /// Returns the description of the argument or `None`.
    pub fn description(&self) -> Option<&'a str> {
        self.description
    }

    /// Returns the default value of the argument or `None`.
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }

    /// Marks the `Arg` as required and returns it.
    pub fn required(mut self, yes: bool) -> Self {
        self.required = yes;
        self
    }

    /// Sets the description for the `Arg` and returns it.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the default value for the `Arg` and returns it.
    pub fn with_default_value<V>(mut self, default_value: V) -> Self
    where
        V: Into<Value>,
    {
        self.default_value = Some(default_value.into());
        self.required = false;
        self
    }

    /// Returns `true` if any of the arg's attributes (e.g. name or description) contains the
    /// keyword. The keyword is assumed to be lowercase.
    ///
    /// This is useful for search.
    pub fn contains_keyword(&self, keyword: &str) -> bool {
        self.name().to_lowercase().contains(keyword)
            || self
                .description()
                .map(|desc| desc.to_lowercase())
                .map(|desc| desc.contains(keyword))
                .unwrap_or(false)
    }
}

impl<'a> fmt::Display for Arg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.default_value {
            Some(value) => write!(f, "{}={}", self.name, value),
            None => f.write_str(self.name),
        }
    }
}

impl<'a> From<&Arg<'a>> for Arg<'a> {
    fn from(arg: &Arg<'a>) -> Self {
        arg.clone()
    }
}

/// Represents a match of a transformation that was parsed from user input.
pub struct DefinitionMatch<'a> {
    name: &'a str,
    args: HashMap<&'a str, ArgMatch<'a>>,
}

impl<'a> DefinitionMatch<'a> {
    /// Creates a new `DefinitionMatch` with the name of the matched `Definition` and the arguments
    /// that where parsed from the input.
    pub fn new<I>(name: &'a str, args: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, ArgMatch<'a>)>,
    {
        DefinitionMatch {
            name,
            args: args.into_iter().collect(),
        }
    }

    /// Returns the name of the matched `Definition`.
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns true if the argument with `name` is present.
    pub fn is_present(&self, name: &str) -> bool {
        self.args.contains_key(name)
    }

    /// Looks up the expression for the argument with `name` from the match.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument contains a value instead of an
    /// expression an error is returned.
    pub fn expr(&self, name: &str) -> Result<&Vec<DefinitionMatch<'a>>> {
        self.arg(name).and_then(|arg_match| match arg_match {
            ArgMatch::Expr(expr) => Ok(expr),
            ArgMatch::Value(_) => Err(self.value_error(name, "expression", "value")),
        })
    }

    /// Looks up the value for the argument with `name` from the match and passes it to the
    /// `map_expr` closure. Returns the value produced by the closure.
    ///
    /// ## Errors
    ///
    /// Returns an error if no argument with `name` was matched, the argument contains a literal
    /// value or if the `map_expr` closure returned an error.
    pub fn map_expr<F, T, E>(&self, name: &str, map_expr: F) -> Result<T>
    where
        F: FnOnce(&[DefinitionMatch<'a>]) -> Result<T, E>,
        E: fmt::Display,
    {
        self.expr(name)
            .and_then(|value| map_expr(value).map_err(|err| self.argument_error(name, err)))
    }

    /// Looks up the value for the argument with `name` from the match.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument contains an expression an error
    /// is returned.
    pub fn value(&self, name: &str) -> Result<&Value> {
        self.arg(name).and_then(|arg_match| match arg_match {
            ArgMatch::Value(value) => Ok(value),
            ArgMatch::Expr(_) => Err(self.value_error(name, "value", "expression")),
        })
    }

    /// Looks up the value for the argument with `name` from the match and passes it to the
    /// `map_value` closure. Returns the value produced by the closure.
    ///
    /// ## Errors
    ///
    /// Returns an error if no argument with `name` was matched, the argument contains an
    /// expression or if the `map_value` closure returned an error.
    pub fn map_value<F, T, E>(&self, name: &str, map_value: F) -> Result<T>
    where
        F: FnOnce(&Value) -> Result<T, E>,
        E: fmt::Display,
    {
        self.value(name)
            .and_then(|value| map_value(value).map_err(|err| self.argument_error(name, err)))
    }

    /// Looks up the value for the argument with `name` from the match and returns it as a `&str`.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument is not a string an error is
    /// returned.
    pub fn str_value(&self, name: &str) -> Result<&str> {
        self.value(name).and_then(|value| match value {
            Value::String(s) => Ok(s.as_str()),
            value => Err(self.value_error(name, "string", value)),
        })
    }

    /// Looks up the value for the argument with `name` from the match and returns it as a `bool`.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument is not a bool an error is
    /// returned.
    pub fn bool_value(&self, name: &str) -> Result<bool> {
        self.value(name).and_then(|value| match value {
            Value::Bool(b) => Ok(*b),
            value => Err(self.value_error(name, "boolean", value)),
        })
    }

    /// Looks up the value for the argument with `name` from the match and returns it as a `Number`.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument is not a number an error is
    /// returned.
    pub fn numeric_value(&self, name: &str) -> Result<&Number> {
        self.value(name).and_then(|value| match value {
            Value::Number(n) => Ok(n),
            value => Err(self.value_error(name, "number", value)),
        })
    }

    /// Looks up the value for the argument with `name` from the match and tries to convert it into
    /// a `T`.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument is not a string an error is
    /// returned.
    pub fn parse_str<T>(&self, name: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        self.str_value(name)
            .and_then(|s| s.parse::<T>().map_err(|err| self.argument_error(name, err)))
    }

    /// Looks up the value for the argument with `name` from the match and returns it as a number
    /// of type `T`.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument is not numeric an error is
    /// returned.
    pub fn parse_number<T>(&self, name: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        self.numeric_value(name).and_then(|n| {
            n.to_string()
                .parse::<T>()
                .map_err(|err| self.argument_error(name, err))
        })
    }

    fn arg(&self, name: &str) -> Result<&ArgMatch<'a>> {
        self.args
            .get(name)
            .ok_or_else(|| Error::new(format!("Argument `{}` missing for `{}`", name, self.name)))
    }

    fn argument_error<E>(&self, name: &str, err: E) -> Error
    where
        E: fmt::Display,
    {
        Error::new(format!(
            "Invalid argument `{}` for `{}`: {}",
            name, self.name, err
        ))
    }

    fn value_error<E, G>(&self, name: &str, expected: E, got: G) -> Error
    where
        E: fmt::Display,
        G: fmt::Display,
    {
        self.argument_error(name, format!("Expected {}, got {}", expected, got))
    }
}

/// Represents a matched transformation function argument.
pub enum ArgMatch<'a> {
    /// A JSON value.
    Value(Value),
    /// An expression composed out of one or more other definition matches.
    Expr(Vec<DefinitionMatch<'a>>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn contains_keyword() {
        let def = Definition::new("foo")
            .add_aliases(&["barbaz", "qux"])
            .with_description("some description")
            .add_arg(Arg::new("the-arg").with_description("this is some argument"));

        assert!(def.contains_keyword("foo"));
        assert!(def.contains_keyword("baz"));
        assert!(def.contains_keyword("arg"));
        assert!(def.contains_keyword("argu"));
        assert!(def.contains_keyword("desc"));
        assert!(!def.contains_keyword("something else"));
    }
}
