use crate::output::{BufferedStdoutPrinter, ColorChoice};
use anyhow::Result;
use dts_core::transform::{
    dsl::{Arg, Definition, DefinitionMatch, Definitions},
    sort::ValueSorter,
    Transformation,
};
use indoc::indoc;
use std::io::{self};
use termcolor::{Color, ColorSpec};

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
                    filter results.
                "#})
                .add_arg(
                    Arg::new("query")
                        .with_description(indoc! {r#"
                            A jsonpath query.

                            See <https://goessner.net/articles/JsonPath/> for supported operators.
                        "#})),
        )
        .add_definition(
            Definition::new("flatten")
                .add_aliases(&["f"])
                .with_description(indoc! {r#"
                    Removes one level of nesting if the data is shaped like an array or
                    one-elemented object. Can be specified multiple times.

                    If the input is a one-elemented array it will be removed entirely, leaving the
                    single element as output.
                "#}),
        )
        .add_definition(
            Definition::new("flatten_keys")
                .add_aliases(&["F", "flatten-keys"])
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
                .add_aliases(&["e", "expand-keys"])
                .with_description("Recursively expands flat object keys to nested objects.")
        )
        .add_definition(
            Definition::new("remove_empty_values")
                .add_aliases(&["r", "remove-empty-values"])
                .with_description(indoc! {r#"
                    Recursively removes nulls, empty arrays and empty objects from the data.

                    Top level empty values are not removed.
                "#})
        )
        .add_definition(
            Definition::new("deep_merge")
                .add_aliases(&["m", "deep-merge"])
                .with_description(indoc! {r#"
                    If the data is an array, all children are merged into one from left to right.
                    Otherwise this is a no-op.

                    Arrays are merged by recurively merging values at the same index. Nulls on the
                    righthand side not merged.

                    Objects are merged by creating a new object with all keys from the left and
                    right value. Keys present on sides are merged recursively.

                    In all other cases, the rightmost value is taken.
                "#})
        )
        .add_definition(
            Definition::new("keys")
                .add_alias("k")
                .with_description(indoc! {r#"
                    Transforms the data into an array of object keys which is empty if the top
                    level value is not an object.
                "#})
        )
        .add_definition(
            Definition::new("delete_keys")
                .add_aliases(&["d", "delete-keys"])
                .with_description(indoc! {r#"
                    Recursively deletes all object keys matching a regex pattern.
                "#})
                .add_arg(
                    Arg::new("pattern")
                        .with_description(indoc! {r#"
                            A regex pattern to match the keys that should be deleted.
                        "#})
                )
        )
        .add_definition(
            Definition::new("sort")
                .add_alias("s")
                .with_description(indoc! {r#"
                    Sorts collections (arrays and maps) recursively.

                    Optionally accepts a `max_depth` which defines the upper bound for child
                    collections to be visited and sorted.

                    If `max_depth` is omitted, the sorter will recursively visit all child
                    collections and sort them.
                "#})
                .add_arg(
                    Arg::new("order")
                        .with_default_value("asc")
                        .with_description(indoc! {r#"
                            The sort order. Possible values are "asc" and "desc".
                        "#})
                )
                .add_arg(
                    Arg::new("max_depth")
                        .required(false)
                        .with_description(indoc! {r#"
                            Defines the upper bound for child collections to be visited and sorted.
                            A max depth of 0 means that only the top level is sorted.
                        "#})
                )
        )
        .add_definition(
            Definition::new("arrays_to_objects")
                .add_aliases(&["ato", "arrays-to-objects"])
                .with_description(indoc! {r#"
                    Recursively transforms all arrays into objects with the array index as key.
                "#})
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
        .map(match_transformation)
        .collect()
}

fn match_transformation(m: &DefinitionMatch<'_>) -> Result<Transformation> {
    let transformation = match m.name() {
        "arrays_to_objects" => Transformation::ArraysToObjects,
        "deep_merge" => Transformation::DeepMerge,
        "delete_keys" => Transformation::DeleteKeys(m.value_of("pattern")?),
        "expand_keys" => Transformation::ExpandKeys,
        "flatten" => Transformation::Flatten,
        "flatten_keys" => Transformation::FlattenKeys(m.value_of("prefix")?),
        "jsonpath" => Transformation::JsonPath(m.value_of("query")?),
        "keys" => Transformation::Keys,
        "remove_empty_values" => Transformation::RemoveEmptyValues,
        "sort" => {
            let order = m.value_of("order")?;
            let max_depth = m.value_of("max_depth").ok();
            let sorter = ValueSorter::new(order, max_depth);
            Transformation::Sort(sorter)
        }
        name => panic!("unmatched transformation `{}`, please file a bug", name),
    };

    Ok(transformation)
}

pub fn print_definitions(choice: ColorChoice) -> io::Result<()> {
    let mut definitions = definitions().into_inner();
    definitions.sort_by(|a, b| a.name().cmp(b.name()));

    let mut printer = BufferedStdoutPrinter::new(choice);

    printer.write_colored(
        ColorSpec::new().set_fg(Some(Color::Yellow)),
        "TRANSFORMATIONS:",
    )?;

    for definition in definitions.iter() {
        printer.write("\n")?;
        printer.write(spaces(4))?;
        printer.write_colored(
            ColorSpec::new().set_fg(Some(Color::Green)),
            definition.to_string(),
        )?;

        if !definition.aliases().is_empty() {
            printer.write(spaces(4))?;
            printer.write(format_aliases(definition))?;
        }

        printer.write("\n")?;

        if let Some(desc) = definition.description() {
            printer.write(format_desc(desc, 8))?;
        }

        for arg in definition.args().values() {
            printer.write("\n")?;
            printer.write(spaces(8))?;
            printer.write_colored(
                ColorSpec::new().set_fg(Some(Color::Green)),
                format!("<{}>", arg.name()),
            )?;
            printer.write("\n")?;

            if let Some(desc) = arg.description() {
                printer.write(format_desc(desc, 12))?;
            }
        }
    }

    printer.print()
}

fn format_aliases(definition: &Definition<'_>) -> String {
    let aliases = definition
        .aliases()
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    format!("[aliases: {}]", aliases)
}

fn format_desc(desc: &str, spaces: usize) -> String {
    let mut desc = indent(desc, spaces);
    if !desc.ends_with('\n') {
        desc.push('\n');
    }
    desc
}

fn indent(s: &str, n: usize) -> String {
    let prefix = spaces(n);
    textwrap::indent(s, &prefix)
}

fn spaces(n: usize) -> String {
    " ".repeat(n)
}
