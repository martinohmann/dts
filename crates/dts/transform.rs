use crate::output::{BufferedStdoutPrinter, ColorChoice};
use anyhow::{anyhow, Result};
use dts_core::transform::{
    dsl::{Arg, Definition, DefinitionMatch, Definitions},
    sort::ValueSorter,
    visitor::{KeyVisitor, ValueVisitor},
    Chain, Delete, DeleteKeys, EachKey, EachValue, FlattenKeys, Insert, KeyIndex, Mutate, Remove,
    ReplaceString, Select, Sort, Transform, Unparameterized, Visit, Wrap,
};
use indoc::indoc;
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;
use termcolor::{Color, ColorSpec};

pub fn definitions<'a>() -> Definitions<'a> {
    let query_arg = Arg::new("query").with_description(indoc! {r#"
        A jsonpath query.

        See <https://goessner.net/articles/JsonPath/> for supported operators.
    "#});

    let expression_arg = Arg::new("expression").with_description(indoc! {r#"
        An expression consisting of one or more transformation functions.
    "#});

    let value_arg = Arg::new("value").with_description(indoc! {r#"
        A JSON value.
    "#});

    let regex_pattern_arg = Arg::new("regex_pattern").with_description(indoc! {r#"
        A regex pattern. See <https://docs.rs/regex/latest/regex/index.html>
        for available features of the regex engine.
    "#});

    let max_depth_arg = Arg::new("max_depth")
        .required(false)
        .with_description(indoc! {r#"
            Defines the upper bound for child collections to be visited. A max depth of 0 means
            that only the top level is visited.
        "#});

    Definitions::new()
        .add_definition(
            Definition::new("select")
                .add_alias("sel")
                .with_description(indoc! {r#"
                    Selects values based on a jsonpath query. Can be specified multiple times to
                    allow starting the filtering from the root element again.

                    When using a jsonpath query, the result will always be shaped like an array with zero or
                    more elements. See `flatten` if you want to remove one level of nesting on single element
                    filter results.
                "#})
                .add_arg(&query_arg),
        )
        .add_definition(
            Definition::new("flatten")
                .with_description(indoc! {r#"
                    Removes one level of nesting if the data is shaped like an array or
                    one-elemented object. Can be specified multiple times.

                    If the input is a one-elemented array it will be removed entirely, leaving the
                    single element as output.
                "#}),
        )
        .add_definition(
            Definition::new("flatten_keys")
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
                .with_description("Recursively expands flat object keys to nested objects.")
        )
        .add_definition(
            Definition::new("remove_empty_values")
                .add_alias("remove_empty")
                .with_description(indoc! {r#"
                    Removes nulls, empty arrays and empty objects.

                    Empty top level values are not removed.
                "#})
        )
        .add_definition(
            Definition::new("deep_merge")
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
                .with_description(indoc! {r#"
                    Transforms the data into an array of object keys which is empty if the top
                    level value is not an object.
                "#})
        )
        .add_definition(
            Definition::new("delete_keys")
                .with_description(indoc! {r#"
                    Deletes object keys matching a regex pattern.
                "#})
                .add_arg(&regex_pattern_arg)
        )
        .add_definition(
            Definition::new("sort")
                .with_description(indoc! {r#"
                    Sorts arrays and objects in the specified order. Objects are sorted by key
                    whereas arrays are sorted by value.
                "#})
                .add_arg(
                    Arg::new("order")
                        .with_default_value("asc")
                        .with_description(indoc! {r#"
                            The sort order. Possible values are "asc" and "desc".
                        "#})
                )
        )
        .add_definition(
            Definition::new("array_to_object")
                .with_description(indoc! {r#"
                    Converts an array into an object with the array indices as keys.
                "#})
        )
        .add_definition(
            Definition::new("mutate")
                .add_alias("mut")
                .with_description(indoc! {r#"
                    Applies the expression to all values matched by the query and returns the
                    mutated value.
                "#})
                .add_args([&query_arg, &expression_arg])
        )
        .add_definition(
            Definition::new("delete")
                .add_alias("del")
                .with_description(indoc! {r#"
                    Selectively deletes values based on a jsonpath query. Deleted values are
                    represented as nulls.
                "#})
                .add_arg(&query_arg),
        )
        .add_definition(
            Definition::new("remove")
                .add_alias("rm")
                .with_description(indoc! {r#"
                    Selectively removes values based on a jsonpath query.
                "#})
                .add_arg(&query_arg),
        )
        .add_definition(
            Definition::new("each_key")
                .with_description(indoc! {r#"
                    Applies the expression to all keys of the current object. This is a no-op for
                    non-object values.
                "#})
                .add_arg(&expression_arg)
        )
        .add_definition(
            Definition::new("each_value")
                .add_alias("each_val")
                .with_description(indoc! {r#"
                    Applies the expression to all values of the current array or object. This is a
                    no-op for non-array and non-object values.
                "#})
                .add_arg(&expression_arg)
        )
        .add_definition(
            Definition::new("values")
                .add_alias("vals")
                .with_description(indoc! {r#"
                    Transforms the data into an array of values which is empty if the top
                    level value is not an array or object.
                "#})
        )
        .add_definition(
            Definition::new("value")
                .add_aliases(&["val", "yield"])
                .with_description(indoc! {r#"
                    Yields a value and discards the old one.

                    This transformation is useful to replace the results of a transformation that
                    selects values based on a jsonpath query with static values, e.g. as argument
                    for `mutate`.
                "#})
                .add_arg(&value_arg)
        )
        .add_definition(
            Definition::new("visit_keys")
                .with_description(indoc! {r#"
                    Recursively visits object keys and applies the expression to them.
                "#})
                .add_args([&expression_arg, &max_depth_arg])
        )
        .add_definition(
            Definition::new("visit_values")
                .add_alias("visit_vals")
                .with_description(indoc! {r#"
                    Recursively visits array and object value and applies the expression to them.
                "#})
                .add_args([&expression_arg, &max_depth_arg])
        )
        .add_definition(
            Definition::new("replace_string")
                .with_description(indoc! {r#"
                    Replaces at most `limit` non-overlapping matches in string values with the
                    replacement provided. If `limit` is 0, then all non-overlapping matches are
                    replaced. For non-string values this is a no-op.
                "#})
                .add_arg(&regex_pattern_arg)
                .add_arg(
                    Arg::new("replacement")
                        .with_description(indoc! {r#"
                            The replacement string, where `$N` and `$name` are expanded to match
                            capture groups.
                        "#})
                )
                .add_arg(
                    Arg::new("limit")
                        .with_default_value(0usize)
                        .with_description(indoc! {r#"
                            The maximum number of non-overlapping matches to replace.
                        "#})
                )
        )
        .add_definition(
            Definition::new("wrap_array")
                .with_description("Wraps a value into an array.")
        )
        .add_definition(
            Definition::new("wrap_object")
                .with_description("Wraps a value into an object with the given key.")
                .add_arg(
                    Arg::new("key")
                        .with_description(indoc! {r#"
                            The key used to insert the value into the new object.
                        "#})
                )
        )
        .add_definition(
            Definition::new("insert")
                .with_description(indoc! {r#"
                    Inserts a value into an array or object.

                    This is a no-op if the value that should be inserted into is not an array or
                    object. If an array index is greater than the array length, the value is
                    appended instead. If an object key already exists, it is overwritten.
                "#})
                .add_arg(
                    Arg::new("key_or_index")
                        .with_description(indoc! {r#"
                            The object key or array index at which the value should be inserted.
                        "#})
                )
                .add_arg(&expression_arg)
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
        "array_to_object" => Box::new(Unparameterized::ArrayToObject),
        "deep_merge" => Box::new(Unparameterized::DeepMerge),
        "delete" => Box::new(Delete::new(m.parse_str("query")?)),
        "delete_keys" => Box::new(DeleteKeys::new(m.parse_str("regex_pattern")?)),
        "each_key" => Box::new(EachKey::new(m.map_expr("expression", parse_matches)?)),
        "each_value" => Box::new(EachValue::new(m.map_expr("expression", parse_matches)?)),
        "expand_keys" => Box::new(Unparameterized::ExpandKeys),
        "flatten" => Box::new(Unparameterized::Flatten),
        "flatten_keys" => Box::new(FlattenKeys::new(m.str_value("prefix")?)),
        "insert" => {
            #[allow(clippy::redundant_closure)]
            let key_or_index = m.map_value("key_or_index", |value| KeyIndex::try_from(value))?;
            let expression = m.map_expr("expression", parse_matches)?;
            Box::new(Insert::new(key_or_index, expression))
        }
        "keys" => Box::new(Unparameterized::Keys),
        "mutate" => {
            let mutator = m.parse_str("query")?;
            let chain = m.map_expr("expression", parse_matches)?;
            Box::new(Mutate::new(mutator, chain))
        }
        "remove" => Box::new(Remove::new(m.parse_str("query")?)),
        "remove_empty_values" => Box::new(Unparameterized::RemoveEmptyValues),
        "replace_string" => {
            let regex = m.parse_str("regex_pattern")?;
            let replacement = m.str_value("replacement")?;
            let limit = m.parse_number("limit")?;
            Box::new(ReplaceString::new(regex, replacement, limit))
        }
        "select" => Box::new(Select::new(m.parse_str("query")?)),
        "sort" => {
            let sorter = ValueSorter::new(m.parse_str("order")?);
            Box::new(Sort::new(sorter))
        }
        "value" => Box::new(m.value("value")?.clone()),
        "values" => Box::new(Unparameterized::Values),
        "visit_keys" => {
            let visitor = KeyVisitor::new(m.map_expr("expression", parse_matches)?);
            let max_depth = parse_optional_number(m, "max_depth")?;
            Box::new(Visit::new(visitor, max_depth))
        }
        "visit_values" => {
            let visitor = ValueVisitor::new(m.map_expr("expression", parse_matches)?);
            let max_depth = parse_optional_number(m, "max_depth")?;
            Box::new(Visit::new(visitor, max_depth))
        }
        "wrap_array" => Box::new(Wrap::Array),
        "wrap_object" => Box::new(Wrap::Object(m.str_value("key")?.to_owned())),
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

        Function arguments may be function expressions or any valid JSON value. Literal strings
        need to be double quoted.
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

fn parse_optional_number<T>(m: &DefinitionMatch, name: &str) -> Result<Option<T>>
where
    T: FromStr,
    <T as FromStr>::Err: fmt::Display,
{
    if m.is_present(name) {
        Ok(Some(m.parse_number(name)?))
    } else {
        Ok(None)
    }
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

    format!("[aliases: {}]", aliases.join(", "))
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
