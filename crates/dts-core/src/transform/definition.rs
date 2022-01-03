//! Provides a DSL to define transformations and parse user input.

use crate::parsers::func_sig::{self, FuncArg};
use crate::{Error, Result};
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
/// use dts_core::transform::{Arg, Definition};
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
        aliases.iter().fold(self, |def, alias| def.add_alias(alias))
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
        args.into_iter().fold(self, |def, arg| def.add_arg(arg))
    }

    // Matches a list of function arguments against the `Definition` and returns a `HashMap` of
    // argument name to argument value or an error if a required argument is missing. Optional
    // arguments receive their default value if they did not appear in `func_args`.
    fn match_func_args(&self, func_args: &[FuncArg<'a>]) -> Result<HashMap<&'a str, &'a str>> {
        let mut remaining_args = self.args.clone();
        let mut args: HashMap<&'a str, &'a str> = HashMap::new();

        for arg in func_args.iter() {
            let (name, value) = match arg {
                FuncArg::Named(name, value) => {
                    let arg_def = remaining_args.shift_remove(name).ok_or_else(|| {
                        Error::new(format!(
                            "Unexpected named argument `{}=\"{}\"`",
                            name, value
                        ))
                    })?;
                    (arg_def.name, value)
                }
                FuncArg::Positional(value) => {
                    let (_, arg_def) = remaining_args.shift_remove_index(0).ok_or_else(|| {
                        Error::new(format!("Unexpected positional argument `{}`", value))
                    })?;
                    (arg_def.name, value)
                }
            };

            if args.contains_key(name) {
                return Err(Error::new(format!("Duplicate argument `{}`", name)));
            }

            args.insert(name, value);
        }

        let mut missing_args = Vec::new();

        for (name, arg_def) in remaining_args.into_iter() {
            match arg_def.default_value() {
                Some(default_value) => {
                    args.insert(name, default_value);
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
/// use dts_core::transform::{Arg, Definition, Definitions};
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

    // Find a definition in the registry. Aliases are used for the lookup as well.
    fn find(&self, name: &'a str) -> Option<&Definition<'a>> {
        for def in self.inner.iter() {
            if def.name == name || def.aliases.iter().any(|&alias| alias == name) {
                return Some(def);
            }
        }

        None
    }

    /// Parses transformations from a `&str` and returns a `Vec<DefinitionMatch>` which contains
    /// the names and resolved arguments of the matching definitions.
    ///
    /// ## Example
    ///
    /// ```
    /// use dts_core::transform::{Arg, Definition, Definitions};
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
    /// let matches = definitions.parse("sort('asc')")?;
    ///
    /// for m in matches {
    ///     match m.name() {
    ///         "sort" => {
    ///             let order: String = m.arg("order")?;
    ///             assert_eq!(&order, "asc");
    ///         }
    ///         _ => unimplemented!()
    ///     }
    /// }
    /// #     Ok(())
    /// # }
    /// ```
    pub fn parse(&self, input: &'a str) -> Result<Vec<DefinitionMatch<'a>>> {
        func_sig::parse(input)?
            .iter()
            .map(|sig| {
                let def = self
                    .find(sig.name())
                    .ok_or_else(|| Error::new(format!("Unknown function `{}`", sig.name())))?;

                let args = def.match_func_args(sig.args()).map_err(|err| {
                    Error::new(format!("Invalid function signature `{}`: {}", sig, err))
                })?;

                Ok(DefinitionMatch::new(def.name, args))
            })
            .collect()
    }
}

/// Represents an argument for a transformation.
///
/// ## Example
///
/// ```
/// use dts_core::transform::Arg;
///
/// let arg = Arg::new("order")
///     .with_description("The sorting order")
///     .with_default_value("asc");
/// ```
#[derive(Default, Clone)]
pub struct Arg<'a> {
    name: &'a str,
    required: bool,
    default_value: Option<&'a str>,
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
    pub fn default_value(&self) -> Option<&'a str> {
        self.default_value
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
    ///
    /// The default value is always a `&str` which may be converted into an appropriate type when
    /// constructing the actual `Transformation`.
    pub fn with_default_value(mut self, default_value: &'a str) -> Self {
        self.default_value = Some(default_value);
        self.required = false;
        self
    }
}

impl<'a> fmt::Display for Arg<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.default_value {
            Some(value) => write!(f, "{}=\"{}\"", self.name, value),
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
    args: HashMap<&'a str, &'a str>,
}

impl<'a> DefinitionMatch<'a> {
    /// Creates a new `DefinitionMatch` with the name of the matched `Definition` and the arguments
    /// that where parsed from the input.
    pub fn new<I>(name: &'a str, args: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
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

    /// Looks up the value for the argument with `name` from the match and attempts to convert it
    /// to `T`.
    ///
    /// ## Errors
    ///
    /// If no argument with `name` was matched or if the argument value is not convertible into `T`
    /// an error is returned.
    pub fn value_of<T>(&self, name: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        self.map_value_of(name, T::from_str)
    }

    /// Looks up the value for the argument with `name` from the match and passes it to the
    /// `map_value` closure. Returns the value produced by the closure.
    ///
    /// ## Errors
    ///
    /// Returns an error if no argument with `name` was matched or if the `map_value` closure
    /// returned an error.
    pub fn map_value_of<F, T, E>(&self, name: &str, map_value: F) -> Result<T>
    where
        F: FnOnce(&str) -> Result<T, E>,
        E: fmt::Display,
    {
        self.args
            .get(name)
            .ok_or_else(|| Error::new(format!("Argument `{}` missing for `{}`", name, self.name)))
            .and_then(|value| {
                map_value(value).map_err(|err| {
                    Error::new(format!(
                        "Invalid argument `{}` for `{}`: {}",
                        name, self.name, err
                    ))
                })
            })
    }
}
