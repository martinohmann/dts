use crate::output::{BufferedStdoutPrinter, ColorChoice};
use anyhow::{anyhow, Result};
use dts_core::transform::{
    dsl::{Arg, Definition, DefinitionMatch, Definitions},
    sort::ValueSorter,
    Chain, Delete, DeleteKeys, EachKey, EachValue, FlattenKeys, Mutate, Remove, Select, Sort,
    Transform, Unparameterized,
};
use indoc::indoc;
use termcolor::{Color, ColorSpec};

pub fn definitions<'a>() -> Definitions<'a> {
    let query_arg = Arg::new("query").with_description(indoc! {r#"
        A jsonpath query.

        See <https://goessner.net/articles/JsonPath/> for supported operators.
    "#});

    let expression_arg = Arg::new("expression").with_description(indoc! {r#"
        An expression consisting of one or more transformation functions.
    "#});

    Definitions::new()
        .add_definition(
            Definition::new("select")
                .add_aliases(&["j", "jp", "jsonpath"])
                .with_description(indoc! {r#"
                    Selects values based on a jsonpath query. Can be specified multiple times to
                    allow starting the filtering from the root element again.

                    When using a jsonpath query, the result will always be shaped like an array with zero or
                    more elements. See `flatten` if you want to remove one level of nesting on single element
                    filter results.
                "#})
                .add_arg(query_arg.clone()),
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
        .add_definition(
            Definition::new("mutate")
                .with_description(indoc! {r#"
                    Applies the expression to all values matched by the query and returns the
                    mutated value.
                "#})
                .add_args(&[query_arg.clone(), expression_arg.clone()])
        )
        .add_definition(
            Definition::new("delete")
                .with_description(indoc! {r#"
                    Selectively deletes values based on a jsonpath query. Deleted values are
                    represented as nulls.
                "#})
                .add_arg(query_arg.clone()),
        )
        .add_definition(
            Definition::new("remove")
                .with_description(indoc! {r#"
                    Selectively removes values based on a jsonpath query.
                "#})
                .add_arg(query_arg),
        )
        .add_definition(
            Definition::new("each_key")
                .with_description(indoc! {r#"
                    Applies the expression to all keys of the current object. This is a no-op for
                    non-object values.
                "#})
                .add_arg(expression_arg.clone())
        )
        .add_definition(
            Definition::new("each_value")
                .with_description(indoc! {r#"
                    Applies the expression to all values of the current array or object. This is a
                    no-op for non-array and non-object values.
                "#})
                .add_arg(expression_arg.clone())
        )
}

/// Parses expressions into a chain of transformations.
pub fn parse_expressions<T>(expressions: &[T]) -> Result<Chain>
where
    T: AsRef<str>,
{
    let definitions = definitions();

    let match_groups = expressions
        .iter()
        .map(|expression| definitions.parse(expression.as_ref()))
        .collect::<Result<Vec<_>, dts_core::Error>>()?;

    let matches = match_groups.into_iter().flatten().collect::<Vec<_>>();

    parse_matches(&matches)
}

fn parse_matches(matches: &[DefinitionMatch<'_>]) -> Result<Chain> {
    matches.iter().map(parse_transformation).collect()
}

fn parse_transformation(m: &DefinitionMatch<'_>) -> Result<Box<dyn Transform>> {
    let transformation: Box<dyn Transform> = match m.name() {
        "arrays_to_objects" => Box::new(Unparameterized::ArraysToObjects),
        "deep_merge" => Box::new(Unparameterized::DeepMerge),
        "delete" => Box::new(Delete::new(m.value_of("query")?)),
        "delete_keys" => Box::new(DeleteKeys::new(m.value_of("pattern")?)),
        "each_key" => Box::new(EachKey::new(m.map_expr_of("expression", parse_matches)?)),
        "each_value" => Box::new(EachValue::new(m.map_expr_of("expression", parse_matches)?)),
        "expand_keys" => Box::new(Unparameterized::ExpandKeys),
        "flatten" => Box::new(Unparameterized::Flatten),
        "flatten_keys" => {
            let prefix: String = m.value_of("prefix")?;
            Box::new(FlattenKeys::new(&prefix))
        }
        "keys" => Box::new(Unparameterized::Keys),
        "mutate" => {
            let mutator = m.value_of("query")?;
            let chain = m.map_expr_of("expression", parse_matches)?;
            Box::new(Mutate::new(mutator, chain))
        }
        "remove" => Box::new(Remove::new(m.value_of("query")?)),
        "remove_empty_values" => Box::new(Unparameterized::RemoveEmptyValues),
        "select" => Box::new(Select::new(m.value_of("query")?)),
        "sort" => {
            let order = m.value_of("order")?;
            let max_depth = m.value_of("max_depth").ok();
            let sorter = ValueSorter::new(order, max_depth);
            Box::new(Sort::new(sorter))
        }
        name => panic!("unmatched transformation `{}`, please file a bug", name),
    };

    Ok(transformation)
}

/// Prints the help for the transformation functions.
pub fn print_transform_help<T>(keywords: &[T], choice: ColorChoice) -> Result<()>
where
    T: AsRef<str>,
{
    let mut definitions = definitions().into_inner();
    definitions.sort_by_key(|definition| definition.name());

    let mut printer = BufferedStdoutPrinter::new(choice);

    if !keywords.is_empty() {
        definitions = filter_definitions(definitions, keywords)?;
    }

    printer.write(indoc! {r#"
        dts provides several transformation functions which are evaluated after the input is
        deserialized into an internal representation that resembles JSON.

        A transformation expression containing one or more transformation functions separated
        either by '.' or spaces.

        Transformation functions may have one of the following forms:

            function_name                    # no parenthesis
            function_name()                  # empty parenthesis
            function_name(arg1)              # single argument
            function_name(arg1, arg2)        # multiple arguments
            function_name(arg2=value, arg1)  # named argument in different position

        Function arguments may be quoted string, numbers, booleans or function expressions.
    "#})?;
    printer.write("\n")?;
    printer.write_colored(ColorSpec::new().set_fg(Some(Color::Yellow)), "EXAMPLE:")?;
    printer.write("\n")?;
    printer.write(indent(
        r#"dts input.json --transform 'jsonpath("$.selector").flatten.sort(order="asc")' -o toml"#,
        4,
    ))?;
    printer.write("\n\n")?;
    printer.write_colored(ColorSpec::new().set_fg(Some(Color::Yellow)), "FUNCTIONS:")?;

    for definition in definitions.iter() {
        printer.write("\n")?;
        printer.write(spaces(4))?;
        printer.write_colored(
            ColorSpec::new().set_fg(Some(Color::Green)),
            definition.to_string(),
        )?;
        printer.write("\n")?;

        if let Some(desc) = definition.description() {
            printer.write(format_desc(desc, 8))?;
        }

        if !definition.aliases().is_empty() {
            printer.write("\n")?;
            printer.write(spaces(8))?;
            printer.write(format_aliases(definition))?;
            printer.write("\n")?;
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

    printer.print()?;

    Ok(())
}

fn filter_definitions<'a, T>(
    definitions: Vec<Definition<'a>>,
    keywords: &[T],
) -> Result<Vec<Definition<'a>>>
where
    T: AsRef<str>,
{
    let keywords = keywords
        .iter()
        .flat_map(|keyword| keyword.as_ref().split_whitespace())
        .map(|keyword| keyword.to_lowercase())
        .collect::<Vec<_>>();

    let filtered = definitions
        .into_iter()
        .filter(|definition| {
            keywords
                .iter()
                .all(|keyword| definition.contains_keyword(keyword))
        })
        .collect::<Vec<Definition>>();

    if filtered.is_empty() {
        return Err(anyhow!("No matches for keywords `{}`", keywords.join(" ")));
    }

    Ok(filtered)
}

fn format_aliases(definition: &Definition) -> String {
    let aliases = Vec::from_iter(definition.aliases().clone());

    format!("[aliases: {}]", aliases.join(","))
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
