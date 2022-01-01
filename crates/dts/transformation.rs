use anyhow::{anyhow, Context, Result};
use dts_core::func_sig::{self, FuncArg};
use dts_core::transform::Transformation;
use indexmap::{IndexMap, IndexSet};
use indoc::indoc;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use textwrap::indent;

#[derive(Default, Clone)]
pub struct Definition<'a> {
    name: &'a str,
    aliases: IndexSet<&'a str>,
    description: Option<&'a str>,
    args: IndexMap<&'a str, Arg<'a>>,
}

impl<'a> Definition<'a> {
    pub fn new(name: &'a str) -> Self {
        Definition {
            name,
            ..Default::default()
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description
    }

    pub fn aliases(&self) -> &IndexSet<&'a str> {
        &self.aliases
    }

    pub fn args(&self) -> &IndexMap<&'a str, Arg<'a>> {
        &self.args
    }

    pub fn add_alias(mut self, alias: &'a str) -> Self {
        self.aliases.insert(alias);
        self
    }

    pub fn add_aliases(self, aliases: &[&'a str]) -> Self {
        aliases.iter().fold(self, |def, alias| def.add_alias(alias))
    }

    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn add_arg<T>(mut self, arg: T) -> Self
    where
        T: Into<Arg<'a>>,
    {
        let arg = arg.into();
        self.args.insert(arg.name, arg);
        // Arguments with default value should always go last.
        self.args
            .sort_by(|_, a, _, b| match (a.default_value, b.default_value) {
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            });
        self
    }

    pub fn add_args<I, T>(self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Arg<'a>>,
    {
        args.into_iter().fold(self, |def, arg| def.add_arg(arg))
    }

    fn match_func_args(&self, func_args: &[FuncArg<'a>]) -> Result<HashMap<&'a str, &'a str>> {
        let mut remaining_args = self.args.clone();
        let mut args: HashMap<&'a str, &'a str> = HashMap::new();

        for arg in func_args.iter() {
            let (name, value) = match arg {
                FuncArg::Named(name, value) => {
                    let arg_def = remaining_args
                        .shift_remove(name)
                        .ok_or_else(|| anyhow!("Unexpected argument `{}=\"{}\"`", name, value))?;
                    (arg_def.name, value)
                }
                FuncArg::Positional(value) => {
                    let (_, arg_def) = remaining_args
                        .shift_remove_index(0)
                        .ok_or_else(|| anyhow!("Unexpected argument `{}`", value))?;
                    (arg_def.name, value)
                }
            };

            if args.contains_key(name) {
                return Err(anyhow!("Duplicate argument `{}`", name));
            }

            args.insert(name, value);
        }

        let mut missing_args = Vec::new();

        for (name, arg_def) in remaining_args.into_iter() {
            match arg_def.default_value {
                Some(default_value) => {
                    args.insert(name, default_value);
                }
                None => {
                    missing_args.push(name);
                }
            }
        }

        if !missing_args.is_empty() {
            return Err(anyhow!(
                "Required arguments missing: {}",
                missing_args.join(","),
            ));
        }

        Ok(args)
    }
}

impl<'a> fmt::Display for Definition<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.args
                .values()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Default)]
pub struct Definitions<'a> {
    inner: Vec<Definition<'a>>,
}

impl<'a> Definitions<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn into_inner(self) -> Vec<Definition<'a>> {
        self.inner
    }

    pub fn add(mut self, trans: Definition<'a>) -> Self {
        self.inner.push(trans);
        self
    }

    fn find(&self, name: &'a str) -> Option<&Definition<'a>> {
        for def in self.inner.iter() {
            if def.name == name || def.aliases.iter().any(|&alias| alias == name) {
                return Some(def);
            }
        }

        None
    }

    pub fn parse(&self, input: &'a str) -> Result<Vec<DefinitionMatch<'a>>> {
        func_sig::parse(input)?
            .iter()
            .map(|sig| {
                let def = self
                    .find(sig.name())
                    .ok_or_else(|| anyhow!("Unknown function `{}`", sig.name()))?;

                let args = def
                    .match_func_args(sig.args())
                    .with_context(|| format!("Invalid function signature `{}`", sig))?;

                Ok(DefinitionMatch::new(def.name, args))
            })
            .collect()
    }
}

#[derive(Default, Clone)]
pub struct Arg<'a> {
    name: &'a str,
    default_value: Option<&'a str>,
    description: Option<&'a str>,
}

impl<'a> Arg<'a> {
    pub fn new(name: &'a str) -> Self {
        Arg {
            name,
            ..Default::default()
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn description(&self) -> Option<&'a str> {
        self.description
    }

    pub fn default_value(&self) -> Option<&'a str> {
        self.default_value
    }

    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_default_value(mut self, default_value: &'a str) -> Self {
        self.default_value = Some(default_value);
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

pub struct DefinitionMatch<'a> {
    name: &'a str,
    args: HashMap<&'a str, &'a str>,
}

impl<'a> DefinitionMatch<'a> {
    pub fn new<I>(name: &'a str, args: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        DefinitionMatch {
            name,
            args: args.into_iter().collect(),
        }
    }

    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn arg<T>(&self, name: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        let arg = self
            .args
            .get(name)
            .ok_or_else(|| anyhow!("Required argument `{}` missing for `{}`", name, self.name))?;

        arg.parse()
            .map_err(|err| anyhow!("{}", err))
            .with_context(|| anyhow!("Invalid argument `{}` for `{}`", name, self.name))
    }
}

pub fn definitions<'a>() -> Definitions<'a> {
    Definitions::new()
        .add(
            Definition::new("jsonpath")
                .add_aliases(&["j", "jp"])
                .with_description(indoc! {r#"
                    Selects data from the decoded input via jsonpath query. Can be specified multiple times to
                    allow starting the filtering from the root element again.
                    
                    When using a jsonpath query, the result will always be shaped like an array with zero or
                    more elements. See `flatten` if you want to remove one level of nesting on single element
                    filter results."#})
                .add_arg(
                    Arg::new("query")
                        .with_description(indoc! {r#"
                            A jsonpath query.

                            See <https://docs.rs/jsonpath-rust/0.1.3/jsonpath_rust/index.html#operators> for supported
                            operators."#})),
        )
        .add(
            Definition::new("flatten")
                .add_aliases(&["f"])
                .with_description(indoc! {r#"
                    Removes one level of nesting if the data is shaped like an array or one-elemented object.
                    Can be specified multiple times.
                    
                    If the input is a one-elemented array it will be removed entirely, leaving the single
                    element as output."#}),
        )
        .add(
            Definition::new("flatten_keys")
                .add_alias("F")
                .with_description(indoc! {r#"
                    Flattens the input to an object with flat keys.
                    
                    The structure of the result is similar to the output of `gron`:
                    <https://github.com/TomNomNom/gron>.
                "#})
                .add_arg(
                    Arg::new("prefix")
                        .with_default_value("data")
                        .with_description("The prefix for flattened keys"))
        )
        .add(
            Definition::new("expand_keys")
                .add_alias("e")
                .with_description("Recursively expands flat object keys to nested objects.")
        )
}

pub fn from_str(input: &str) -> Result<Transformation> {
    let chain = definitions()
        .parse(input)?
        .into_iter()
        .map(|m| match m.name() {
            "flatten" => Ok(Transformation::Flatten),
            "flatten_keys" => Ok(Transformation::FlattenKeys(Some(m.arg("prefix")?))),
            "expand_keys" => Ok(Transformation::ExpandKeys),
            "jsonpath" => Ok(Transformation::JsonPath(m.arg("query")?)),
            _ => unreachable!(),
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(Transformation::Chain(chain))
}

pub fn print_transformations() {
    let defs = definitions();

    let mut s = String::new();

    let mut defs = defs.into_inner();
    defs.sort_by(|a, b| a.name().cmp(b.name()));

    for (i, def) in defs.iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }

        s.push_str(&def.to_string());

        if !def.aliases().is_empty() {
            s.push_str("    [aliases: ");
            s.push_str(
                &def.aliases()
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            );
            s.push(']');
        }

        s.push('\n');

        if let Some(description) = def.description() {
            s.push_str(&indent(description, "    "));
            if !description.ends_with('\n') {
                s.push('\n');
            }
        }

        for arg in def.args().values() {
            s.push('\n');
            s.push_str(&format!("    <{}>\n", arg.name()));
            if let Some(description) = arg.description() {
                s.push_str(&indent(description, "        "));
                if !description.ends_with('\n') {
                    s.push('\n');
                }
            }
        }
    }

    print!("Available transformations:\n\n{}", indent(&s, "    "));
}
