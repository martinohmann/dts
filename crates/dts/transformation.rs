use anyhow::{anyhow, Context, Result};
use indoc::indoc;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use textwrap::indent;

#[derive(Default)]
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

    pub fn aliases(mut self, aliases: &[&'a str]) -> Self {
        self.aliases = aliases.to_vec();
        self
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

    pub fn generate_docs(&self) -> String {
        let mut s = String::new();

        for (i, def) in self.inner.iter().enumerate() {
            if i > 0 {
                s.push('\n');
            }

            s.push_str(def.name);
            s.push('(');

            let args = def
                .args
                .iter()
                .map(|arg| match arg.default_value {
                    Some(value) => format!("{} = \"{}\"", arg.name, value),
                    None => arg.name.to_string(),
                })
                .collect::<Vec<String>>()
                .join(", ");

            s.push_str(&args);
            s.push_str(")\n");

            if let Some(description) = def.description {
                s.push_str(&indent(description, "    "));
                if !description.ends_with('\n') {
                    s.push('\n');
                }
            }

            for arg in def.args.iter() {
                s.push_str(&format!("    <{}>\n", arg.name));
                if let Some(description) = arg.description {
                    s.push_str(&indent(description, "        "));
                    if !description.ends_with('\n') {
                        s.push('\n');
                    }
                }
            }
        }

        s
    }

    pub fn parse(&self, _s: &str) -> Result<Vec<Match>> {
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
            Definition::new("flatten")
                .aliases(&["f"])
                .description(indoc! {r#"
                    Removes one level of nesting if the data is shaped like an array or one-elemented object.
                    Can be specified multiple times.
                    
                    If the input is a one-elemented array it will be removed entirely, leaving the single
                    element as output.
                "#}),
        )
        .add(Definition::new("flatten_keys").arg(Arg::new("prefix").default_value("data").description("prefix for flattened keys")))
        .add(
            Definition::new("jsonpath")
                .aliases(&["j", "jp"])
                .arg(Arg::new("query")),
        )
}

pub fn print_transformations() {
    let defs = definitions();

    print!(
        "Available transformations:\n\n{}",
        indent(&defs.generate_docs(), "    ")
    );
}
