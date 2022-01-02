use anyhow::Result;
use dts_core::transform::{Arg, Definition, Definitions, Transformation};
use indoc::indoc;
use textwrap::indent;

pub fn definitions<'a>() -> Definitions<'a> {
    Definitions::new()
        .add_definition(
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
        .add_definition(
            Definition::new("flatten")
                .add_aliases(&["f"])
                .with_description(indoc! {r#"
                    Removes one level of nesting if the data is shaped like an array or one-elemented object.
                    Can be specified multiple times.
                    
                    If the input is a one-elemented array it will be removed entirely, leaving the single
                    element as output."#}),
        )
        .add_definition(
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
        .add_definition(
            Definition::new("expand_keys")
                .add_alias("e")
                .with_description("Recursively expands flat object keys to nested objects.")
        )
}

pub fn parse_inputs<T>(inputs: &[T]) -> Result<Vec<Transformation>>
where
    T: AsRef<str>,
{
    let definitions = definitions();

    let match_groups = inputs
        .iter()
        .map(|input| definitions.parse(input.as_ref()))
        .collect::<Result<Vec<_>, dts_core::Error>>()?;

    match_groups
        .iter()
        .flatten()
        .map(|m| match m.name() {
            "flatten" => Ok(Transformation::Flatten),
            "flatten_keys" => Ok(Transformation::FlattenKeys(Some(m.arg("prefix")?))),
            "expand_keys" => Ok(Transformation::ExpandKeys),
            "jsonpath" => Ok(Transformation::JsonPath(m.arg("query")?)),
            _ => unreachable!(),
        })
        .collect()
}

pub fn print_definitions() {
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
