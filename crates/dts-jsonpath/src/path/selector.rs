use super::{Filter, Index, SliceRange};
use dts_json::Value;

pub trait PathSelector<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value>;
}

pub trait PathVisitor<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value);
}

pub struct Visitor<'a, F> {
    chain: Vec<JsonPath<'a>>,
    mutate: &'a mut F,
}

impl<'a, F> Visitor<'a, F>
where
    F: FnMut(&mut Value),
{
    pub(crate) fn new<I>(chain: I, mutate: &'a mut F) -> Self
    where
        I: IntoIterator<Item = &'a JsonPath<'a>>,
    {
        Visitor {
            chain: chain.into_iter().cloned().collect(),
            mutate,
        }
    }

    pub(crate) fn visit(&mut self, value: &mut Value) {
        match self.chain.get(0) {
            Some(path) => {
                let mut visitor = Visitor::new(&self.chain[1..], self.mutate);
                path.visit(value, &mut visitor);
            }
            None => (self.mutate)(value),
        }
    }
}

impl<'a, T> PathVisitor<'a> for Box<T>
where
    T: PathVisitor<'a> + ?Sized,
{
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        (**self).visit(value, visitor)
    }
}

impl<'a, T> PathVisitor<'a> for &T
where
    T: PathVisitor<'a> + ?Sized,
{
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        (*self).visit(value, visitor)
    }
}

impl<'a, T> PathSelector<'a> for Box<T>
where
    T: PathSelector<'a> + ?Sized,
{
    fn select(&self, valur: &'a Value) -> Vec<&'a Value> {
        (**self).select(valur)
    }
}

impl<'a, T> PathSelector<'a> for &T
where
    T: PathSelector<'a> + ?Sized,
{
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        (*self).select(value)
    }
}

#[derive(Clone)]
pub enum JsonPath<'a> {
    Root(RootSelector<'a>),
    Current(CurrentSelector),
    Key(KeySelector),
    Wildcard(WildcardSelector),
    Index(IndexSelector),
    Union(UnionSelector<'a>),
    Slice(SliceSelector),
    Descendant(DescendantSelector<'a>),
    Filter(FilterSelector<'a>),
    Chain(ChainSelector<'a>),
}

impl<'a> PathSelector<'a> for JsonPath<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        match self {
            JsonPath::Root(s) => s.select(value),
            JsonPath::Current(s) => s.select(value),
            JsonPath::Key(s) => s.select(value),
            JsonPath::Wildcard(s) => s.select(value),
            JsonPath::Index(s) => s.select(value),
            JsonPath::Union(s) => s.select(value),
            JsonPath::Slice(s) => s.select(value),
            JsonPath::Descendant(s) => s.select(value),
            JsonPath::Filter(s) => s.select(value),
            JsonPath::Chain(s) => s.select(value),
        }
    }
}

impl<'a> PathVisitor<'a> for JsonPath<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match self {
            JsonPath::Root(v) => v.visit(value, visitor),
            JsonPath::Current(v) => v.visit(value, visitor),
            JsonPath::Key(v) => v.visit(value, visitor),
            JsonPath::Wildcard(v) => v.visit(value, visitor),
            JsonPath::Index(v) => v.visit(value, visitor),
            JsonPath::Union(v) => v.visit(value, visitor),
            JsonPath::Slice(v) => v.visit(value, visitor),
            JsonPath::Descendant(v) => v.visit(value, visitor),
            JsonPath::Filter(v) => v.visit(value, visitor),
            JsonPath::Chain(v) => v.visit(value, visitor),
        }
    }
}

#[derive(Clone)]
pub struct ChainSelector<'a> {
    chain: Vec<JsonPath<'a>>,
}

impl<'a> ChainSelector<'a> {
    pub(crate) fn new<I>(chain: I) -> Self
    where
        I: IntoIterator<Item = JsonPath<'a>>,
    {
        ChainSelector {
            chain: chain.into_iter().collect(),
        }
    }
}

impl<'a> FromIterator<JsonPath<'a>> for ChainSelector<'a> {
    fn from_iter<I: IntoIterator<Item = JsonPath<'a>>>(iter: I) -> Self {
        ChainSelector::new(iter)
    }
}

impl<'a> IntoIterator for ChainSelector<'a> {
    type Item = JsonPath<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chain.into_iter()
    }
}

impl<'a> PathSelector<'a> for ChainSelector<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        self.chain.iter().fold(vec![value], |acc, selector| {
            acc.iter()
                .flat_map(|value| selector.select(value))
                .collect()
        })
    }
}

impl<'a> PathVisitor<'a> for ChainSelector<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        self.chain
            .iter()
            .cloned()
            .rev()
            .for_each(|path| visitor.chain.insert(0, path));

        visitor.visit(value);
    }
}

#[derive(Clone)]
pub struct RootSelector<'a> {
    root: &'a Value,
}

impl<'a> RootSelector<'a> {
    pub(crate) fn new(root: &'a Value) -> Self {
        RootSelector { root }
    }
}

impl<'a> PathSelector<'a> for RootSelector<'a> {
    fn select(&self, _value: &'a Value) -> Vec<&'a Value> {
        vec![self.root]
    }
}

impl<'a> PathVisitor<'a> for RootSelector<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        visitor.visit(value);
    }
}

#[derive(Clone)]
pub struct CurrentSelector;

impl<'a> PathSelector<'a> for CurrentSelector {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        vec![value]
    }
}

impl<'a> PathVisitor<'a> for CurrentSelector {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        visitor.visit(value);
    }
}

#[derive(Clone)]
pub struct KeySelector {
    key: String,
}

impl KeySelector {
    pub(crate) fn new(key: String) -> Self {
        KeySelector { key }
    }
}

impl<'a> PathSelector<'a> for KeySelector {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        value
            .as_object()
            .and_then(|object| object.get(&self.key))
            .map(|value| vec![value])
            .unwrap_or_default()
    }
}

impl<'a> PathVisitor<'a> for KeySelector {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(object) = value.as_object_mut() {
            if let Some(value) = object.get_mut(&self.key) {
                visitor.visit(value);
            }
        }
    }
}

#[derive(Clone)]
pub struct WildcardSelector;

impl<'a> PathSelector<'a> for WildcardSelector {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        match value {
            Value::Array(array) => array.iter().collect(),
            Value::Object(object) => object.values().collect(),
            _ => vec![],
        }
    }
}

impl<'a> PathVisitor<'a> for WildcardSelector {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match value {
            Value::Array(array) => array.iter_mut().for_each(|value| visitor.visit(value)),
            Value::Object(object) => object.values_mut().for_each(|value| visitor.visit(value)),
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct IndexSelector {
    index: Index,
}

impl IndexSelector {
    pub(crate) fn new(index: i64) -> Self {
        IndexSelector {
            index: Index::new(index),
        }
    }
}

impl<'a> PathSelector<'a> for IndexSelector {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        value
            .as_array()
            .and_then(|array| {
                self.index
                    .get(array.len() as i64)
                    .map(|index| vec![&array[index]])
            })
            .unwrap_or_default()
    }
}

impl<'a> PathVisitor<'a> for IndexSelector {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(array) = value.as_array_mut() {
            if let Some(index) = self.index.get(array.len() as i64) {
                visitor.visit(&mut array[index]);
            }
        }
    }
}

#[derive(Clone)]
pub struct UnionSelector<'a> {
    entries: Vec<JsonPath<'a>>,
}

impl<'a> UnionSelector<'a> {
    pub(crate) fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = JsonPath<'a>>,
    {
        UnionSelector {
            entries: entries.into_iter().collect(),
        }
    }
}

impl<'a> FromIterator<JsonPath<'a>> for UnionSelector<'a> {
    fn from_iter<I: IntoIterator<Item = JsonPath<'a>>>(iter: I) -> Self {
        UnionSelector::new(iter)
    }
}

impl<'a> PathSelector<'a> for UnionSelector<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        self.entries
            .iter()
            .flat_map(|selector| selector.select(value))
            .collect()
    }
}

impl<'a> PathVisitor<'a> for UnionSelector<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        for entry in self.entries.iter() {
            entry.visit(value, visitor)
        }
    }
}

#[derive(Clone)]
pub struct SliceSelector {
    range: SliceRange,
}

impl SliceSelector {
    pub(crate) fn new(range: SliceRange) -> Self {
        SliceSelector { range }
    }
}

impl<'a> PathSelector<'a> for SliceSelector {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        match value.as_array() {
            Some(array) => {
                let (lower, upper) = self.range.bounds(array.len() as i64);

                match self.range.step() {
                    step @ 1..=i64::MAX => (lower..upper)
                        .step_by(step as usize)
                        .map(|i| &array[i as usize])
                        .collect(),
                    step @ i64::MIN..=-1 => (lower + 1..=upper)
                        .rev()
                        .step_by(-step as usize)
                        .map(|i| &array[i as usize])
                        .collect(),
                    0 => vec![],
                }
            }
            None => vec![],
        }
    }
}

impl<'a> PathVisitor<'a> for SliceSelector {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(array) = value.as_array_mut() {
            let (lower, upper) = self.range.bounds(array.len() as i64);

            match self.range.step() {
                step @ 1..=i64::MAX => (lower..upper)
                    .step_by(step as usize)
                    .for_each(|i| visitor.visit(&mut array[i as usize])),
                step @ i64::MIN..=-1 => (lower + 1..=upper)
                    .rev()
                    .step_by(-step as usize)
                    .for_each(|i| visitor.visit(&mut array[i as usize])),
                0 => (),
            }
        }
    }
}

#[derive(Clone)]
pub struct DescendantSelector<'a> {
    selector: Box<JsonPath<'a>>,
}

impl<'a> DescendantSelector<'a> {
    pub(crate) fn new(selector: JsonPath<'a>) -> Self {
        DescendantSelector {
            selector: Box::new(selector),
        }
    }
}

impl<'a> PathSelector<'a> for DescendantSelector<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        let mut values = self.selector.select(value);

        match value {
            Value::Array(array) => {
                let mut children = array
                    .iter()
                    .flat_map(|value| self.selector.select(value))
                    .collect();
                values.append(&mut children);
                values
            }
            Value::Object(object) => {
                let mut children = object
                    .values()
                    .flat_map(|value| self.selector.select(value))
                    .collect();
                values.append(&mut children);
                values
            }
            _ => values,
        }
    }
}

impl<'a> PathVisitor<'a> for DescendantSelector<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        self.selector.visit(value, visitor);

        match value {
            Value::Array(array) => array.iter_mut().for_each(|value| visitor.visit(value)),
            Value::Object(object) => object.values_mut().for_each(|value| visitor.visit(value)),
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct FilterSelector<'a> {
    filter: Box<Filter<'a>>,
}

impl<'a> FilterSelector<'a> {
    pub(crate) fn new(filter: Filter<'a>) -> Self {
        FilterSelector {
            filter: Box::new(filter),
        }
    }
}

impl<'a> PathSelector<'a> for FilterSelector<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        match value {
            Value::Array(array) => array
                .iter()
                .filter(|value| self.filter.matches(value))
                .collect(),
            Value::Object(object) => object
                .values()
                .filter(|value| self.filter.matches(value))
                .collect(),
            _ => vec![],
        }
    }
}

impl<'a> PathVisitor<'a> for FilterSelector<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match value {
            Value::Array(array) => array
                .iter_mut()
                .filter(|value| self.filter.matches(value))
                .for_each(|value| visitor.visit(value)),
            Value::Object(object) => object
                .values_mut()
                .filter(|value| self.filter.matches(value))
                .for_each(|value| visitor.visit(value)),
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[track_caller]
    fn assert_selects<'a, T>(selector: T, root: &'a Value, expected: Vec<&'a Value>)
    where
        T: PathSelector<'a>,
    {
        assert_eq!(selector.select(root), expected);
    }

    #[test]
    fn test_index_selector() {
        let selector = IndexSelector::new(0);
        assert_selects(selector, &json!([1, 2]), vec![&json!(1)]);

        let selector = IndexSelector::new(-2);
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2)]);
    }

    #[test]
    fn test_slice_selector() {
        let selector = SliceSelector::new(SliceRange::default());
        assert_selects(
            selector,
            &json!([1, 2, 3]),
            vec![&json!(1), &json!(2), &json!(3)],
        );

        let selector = SliceSelector::new(SliceRange::new(Some(1), None, None));
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2), &json!(3)]);

        let selector = SliceSelector::new(SliceRange::new(Some(1), Some(3), None));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5]),
            vec![&json!(2), &json!(3)],
        );

        let selector = SliceSelector::new(SliceRange::new(Some(1), Some(5), Some(2)));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5]),
            vec![&json!(2), &json!(4)],
        );

        let selector = SliceSelector::new(SliceRange::new(Some(5), Some(1), Some(-2)));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5, 6]),
            vec![&json!(6), &json!(4)],
        );

        let selector = SliceSelector::new(SliceRange::new(None, None, Some(-1)));
        assert_selects(
            selector,
            &json!([1, 2, 3]),
            vec![&json!(3), &json!(2), &json!(1)],
        );

        let selector = SliceSelector::new(SliceRange::new(Some(-2), Some(-1), None));
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2)]);

        let selector = SliceSelector::new(SliceRange::new(Some(10), Some(12), None));
        assert_selects(selector, &json!([1, 2, 3]), vec![]);
    }

    #[test]
    fn test_key_selector() {
        let selector = KeySelector::new("foo".into());
        assert_selects(selector, &json!({"foo": 1, "bar": 2}), vec![&json!(1)]);
    }
}
