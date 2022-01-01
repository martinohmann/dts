use anyhow::{anyhow, Context, Result};
use dts_core::funcs::{self, Func, FuncArg};
use indoc::indoc;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use textwrap::indent;

#[derive(Default, Clone)]
pub struct Definition<'a> {
    name: &'a str,
    aliases: Vec<&'a str>,
    description: Option<&'a str>,
    args: Vec<Arg<'a>>,
}

impl<'a> Definition<'a> {
    pub fn new(name: &'a str) -> Self {
        Definition {
            name,
            ..Default::default()
        }
    }

    pub fn alias(mut self, alias: &'a str) -> Self {
        self.aliases.push(alias);
        self
    }

    pub fn aliases(self, aliases: &[&'a str]) -> Self {
        aliases
            .into_iter()
            .fold(self, |def, alias| def.alias(alias))
    }

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn arg<T>(mut self, arg: T) -> Self
    where
        T: Into<Arg<'a>>,
    {
        self.args.push(arg.into());
        // Arguments with default value should always go last.
        self.args
            .sort_by(|a, b| match (a.default_value, b.default_value) {
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (_, _) => Ordering::Equal,
            });
        self
    }

    pub fn args<I, T>(self, args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Arg<'a>>,
    {
        args.into_iter().fold(self, |def, arg| def.arg(arg))
    }

    pub fn format(&self) -> String {
        let mut s = String::new();
        s.push_str(self.name);
        s.push('(');

        let args = self
            .args
            .iter()
            .map(Arg::format)
            .collect::<Vec<_>>()
            .join(", ");

        s.push_str(&args);
        s.push(')');
        if !self.aliases.is_empty() {
            s.push_str("    [aliases: ");
            s.push_str(&self.aliases.join(", "));
            s.push(']');
        }

        s.push('\n');

        if let Some(description) = self.description {
            s.push_str(&indent(description, "    "));
            if !description.ends_with('\n') {
                s.push('\n');
            }
        }

        for arg in self.args.iter() {
            s.push('\n');
            s.push_str(&format!("    <{}>\n", arg.name));
            if let Some(description) = arg.description {
                s.push_str(&indent(description, "        "));
                if !description.ends_with('\n') {
                    s.push('\n');
                }
            }
        }

        s
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

    pub fn add(mut self, trans: Definition<'a>) -> Self {
        self.inner.push(trans);
        self
    }

    pub fn format(&self) -> String {
        let mut s = String::new();

        let mut defs = self.inner.clone();
        defs.sort_by(|a, b| a.name.cmp(&b.name));

        for (i, def) in defs.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }

            s.push_str(&def.format());
        }

        s
    }

    fn find(&self, name: &str) -> Option<&Definition> {
        for def in self.inner.iter() {
            if def.name == name || def.aliases.iter().any(|&alias| alias == name) {
                return Some(def);
            }
        }

        None
    }

    pub fn parse(&self, input: &str) -> Result<Vec<Match>> {
        for func in funcs::parse(input)? {
            let def = match self.find(func.name) {
                Some(def) => def.clone(),
                None => return Err(anyhow!("unknown function `{}`", func.name)),
            };
        }

        Ok(Vec::new())
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

    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    pub fn default_value(mut self, default_value: &'a str) -> Self {
        self.default_value = Some(default_value);
        self
    }

    pub fn format(&self) -> String {
        match self.default_value {
            Some(value) => format!("{} = \"{}\"", self.name, value),
            None => self.name.to_string(),
        }
    }
}

impl<'a> From<&Arg<'a>> for Arg<'a> {
    fn from(arg: &Arg<'a>) -> Self {
        arg.clone()
    }
}

pub struct Match<'a> {
    name: &'a str,
    args: HashMap<&'a str, &'a str>,
}

impl<'a> Match<'a> {
    pub fn new<I>(name: &'a str, args: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        Match {
            name,
            args: args.into_iter().collect(),
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn arg<T>(&self, name: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        let arg = self.args.get(name).ok_or_else(|| {
            anyhow!(
                "required argument `{}` missing for transformation `{}`",
                name,
                self.name
            )
        })?;

        FromStr::from_str(arg)
            .map_err(|err| anyhow!("{}", err))
            .with_context(|| {
                anyhow!(
                    "invalid argument `{}` for transformation `{}`",
                    name,
                    self.name
                )
            })
    }
}

pub fn definitions<'a>() -> Definitions<'a> {
    Definitions::new()
        .add(
            Definition::new("jsonpath")
                .aliases(&["j", "jp"])
                .description(indoc! {r#"
                    Selects data from the decoded input via jsonpath query. Can be specified multiple times to
                    allow starting the filtering from the root element again.
                    
                    When using a jsonpath query, the result will always be shaped like an array with zero or
                    more elements. See `flatten` if you want to remove one level of nesting on single element
                    filter results."#})
                .arg(
                    Arg::new("query")
                        .description(indoc! {r#"
                            A jsonpath query.

                            See <https://docs.rs/jsonpath-rust/0.1.3/jsonpath_rust/index.html#operators> for supported
                            operators."#})),
        )
        .add(
            Definition::new("flatten")
                .aliases(&["f"])
                .description(indoc! {r#"
                    Removes one level of nesting if the data is shaped like an array or one-elemented object.
                    Can be specified multiple times.
                    
                    If the input is a one-elemented array it will be removed entirely, leaving the single
                    element as output."#}),
        )
        .add(
            Definition::new("flatten_keys")
                .alias("F")
                .description(indoc! {r#"
                    Flattens the input to an object with flat keys.
                    
                    The structure of the result is similar to the output of `gron`:
                    <https://github.com/TomNomNom/gron>.
                "#})
                .arg(
                    Arg::new("prefix")
                        .default_value("data")
                        .description("The prefix for flattened keys"))
        )
        .add(
            Definition::new("expand_keys")
                .alias("e")
                .description("Recursively expands flat object keys to nested objects.")
        )
}

pub fn print_transformations() {
    let defs = definitions();

    print!(
        "Available transformations:\n\n{}",
        indent(&defs.format(), "    ")
    );
}
