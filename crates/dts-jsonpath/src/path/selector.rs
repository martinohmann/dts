use super::filter::Filter;
use dts_json::Value;

pub trait PathSelector {
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value>;
}

pub trait PathVisitor {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value);
}

pub struct Visitor<'a, F> {
    chain: Vec<JsonPath>,
    mutate: &'a mut F,
}

impl<'a, F> Visitor<'a, F>
where
    F: FnMut(&mut Value),
{
    pub(crate) fn new<I>(chain: I, mutate: &'a mut F) -> Self
    where
        I: IntoIterator<Item = &'a JsonPath>,
    {
        Visitor {
            chain: chain.into_iter().cloned().collect(),
            mutate,
        }
    }

    pub(crate) fn visit<'v>(&mut self, root: &mut Value, current: &mut Value) {
        match self.chain.get(0) {
            Some(path) => {
                let mut visitor = Visitor::new(&self.chain[1..], self.mutate);
                path.visit(root, current, &mut visitor);
            }
            None => (self.mutate)(current),
        }
    }
}

impl<T> PathVisitor for Box<T>
where
    T: PathVisitor + ?Sized,
{
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        (**self).visit(root, current, visitor)
    }
}

impl<T> PathVisitor for &T
where
    T: PathVisitor + ?Sized,
{
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        (*self).visit(root, current, visitor)
    }
}

impl<T> PathSelector for Box<T>
where
    T: PathSelector + ?Sized,
{
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        (**self).select(root, current)
    }
}

impl<T> PathSelector for &T
where
    T: PathSelector + ?Sized,
{
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        (*self).select(root, current)
    }
}

#[derive(Clone)]
pub enum JsonPath {
    Root(RootSelector),
    Current(CurrentSelector),
    Key(KeySelector),
    Wildcard(WildcardSelector),
    Index(IndexSelector),
    Union(UnionSelector),
    Slice(SliceSelector),
    Descendant(DescendantSelector),
    Filter(FilterSelector),
    Chain(ChainSelector),
}

impl PathSelector for JsonPath {
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        match self {
            JsonPath::Root(s) => s.select(root, current),
            JsonPath::Current(s) => s.select(root, current),
            JsonPath::Key(s) => s.select(root, current),
            JsonPath::Wildcard(s) => s.select(root, current),
            JsonPath::Index(s) => s.select(root, current),
            JsonPath::Union(s) => s.select(root, current),
            JsonPath::Slice(s) => s.select(root, current),
            JsonPath::Descendant(s) => s.select(root, current),
            JsonPath::Filter(s) => s.select(root, current),
            JsonPath::Chain(s) => s.select(root, current),
        }
    }
}

impl PathVisitor for JsonPath {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match self {
            JsonPath::Root(v) => v.visit(root, current, visitor),
            JsonPath::Current(v) => v.visit(root, current, visitor),
            JsonPath::Key(v) => v.visit(root, current, visitor),
            JsonPath::Wildcard(v) => v.visit(root, current, visitor),
            JsonPath::Index(v) => v.visit(root, current, visitor),
            JsonPath::Union(v) => v.visit(root, current, visitor),
            JsonPath::Slice(v) => v.visit(root, current, visitor),
            JsonPath::Descendant(v) => v.visit(root, current, visitor),
            JsonPath::Filter(v) => v.visit(root, current, visitor),
            JsonPath::Chain(v) => v.visit(root, current, visitor),
        }
    }
}

#[derive(Clone)]
pub struct ChainSelector {
    chain: Vec<JsonPath>,
}

impl ChainSelector {
    pub(crate) fn new<I>(chain: I) -> Self
    where
        I: IntoIterator<Item = JsonPath>,
    {
        ChainSelector {
            chain: chain.into_iter().collect(),
        }
    }
}

impl FromIterator<JsonPath> for ChainSelector {
    fn from_iter<I: IntoIterator<Item = JsonPath>>(iter: I) -> Self {
        ChainSelector::new(iter)
    }
}

impl<'a> IntoIterator for ChainSelector {
    type Item = JsonPath;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chain.into_iter()
    }
}

impl PathSelector for ChainSelector {
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        self.chain.iter().fold(vec![current], |acc, selector| {
            acc.iter()
                .flat_map(|value| selector.select(root, value))
                .collect()
        })
    }
}

impl PathVisitor for ChainSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        for path in self.chain.iter().rev() {
            visitor.chain.insert(0, path.clone());
        }

        visitor.visit(root, current);
    }
}

#[derive(Clone)]
pub struct RootSelector;

impl PathSelector for RootSelector {
    fn select<'a>(&self, root: &'a Value, _current: &'a Value) -> Vec<&'a Value> {
        vec![root]
    }
}

impl PathVisitor for RootSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        visitor.visit(root, current);
    }
}

#[derive(Clone)]
pub struct CurrentSelector;

impl PathSelector for CurrentSelector {
    fn select<'a>(&self, _root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        vec![current]
    }
}

impl PathVisitor for CurrentSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        visitor.visit(root, current);
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

impl PathSelector for KeySelector {
    fn select<'a>(&self, _root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        current
            .as_object()
            .and_then(|object| object.get(&self.key))
            .map(|value| vec![value])
            .unwrap_or_default()
    }
}

impl PathVisitor for KeySelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(object) = current.as_object_mut() {
            if let Some(value) = object.get_mut(&self.key) {
                visitor.visit(root, value);
            }
        }
    }
}

#[derive(Clone)]
pub struct WildcardSelector;

impl PathSelector for WildcardSelector {
    fn select<'a>(&self, _root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        match current {
            Value::Array(array) => array.iter().collect(),
            Value::Object(object) => object.values().collect(),
            _ => vec![],
        }
    }
}

impl PathVisitor for WildcardSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match current {
            Value::Array(array) => array
                .iter_mut()
                .for_each(|value| visitor.visit(root, value)),
            Value::Object(object) => object
                .values_mut()
                .for_each(|value| visitor.visit(root, value)),
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct IndexSelector {
    index: i64,
}

impl IndexSelector {
    pub(crate) fn new(index: i64) -> Self {
        IndexSelector { index }
    }

    fn index(&self, len: i64) -> Option<usize> {
        let index = if self.index < 0 {
            len + self.index
        } else {
            self.index
        };

        if index < 0 || index >= len {
            None
        } else {
            Some(index as usize)
        }
    }
}

impl PathSelector for IndexSelector {
    fn select<'a>(&self, _root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        current
            .as_array()
            .and_then(|array| {
                self.index(array.len() as i64)
                    .map(|index| vec![&array[index]])
            })
            .unwrap_or_default()
    }
}

impl PathVisitor for IndexSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(array) = current.as_array_mut() {
            if let Some(index) = self.index(array.len() as i64) {
                visitor.visit(root, &mut array[index]);
            }
        }
    }
}

#[derive(Clone)]
pub struct UnionSelector {
    entries: Vec<JsonPath>,
}

impl UnionSelector {
    pub(crate) fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = JsonPath>,
    {
        UnionSelector {
            entries: entries.into_iter().collect(),
        }
    }
}

impl FromIterator<JsonPath> for UnionSelector {
    fn from_iter<I: IntoIterator<Item = JsonPath>>(iter: I) -> Self {
        UnionSelector::new(iter)
    }
}

impl PathSelector for UnionSelector {
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        self.entries
            .iter()
            .flat_map(|selector| selector.select(root, current))
            .collect()
    }
}

impl PathVisitor for UnionSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        for entry in self.entries.iter() {
            entry.visit(root, current, visitor)
        }
    }
}

#[derive(Default, Clone)]
pub struct SliceRange {
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub step: Option<i64>,
}

impl SliceRange {
    pub(crate) fn new(start: Option<i64>, end: Option<i64>, step: Option<i64>) -> Self {
        SliceRange { start, end, step }
    }

    fn start(&self, len: i64) -> i64 {
        match self.start {
            Some(start) => start,
            None => {
                if self.step() >= 0 {
                    0
                } else {
                    len - 1
                }
            }
        }
    }

    fn end(&self, len: i64) -> i64 {
        match self.end {
            Some(end) => end,
            None => {
                if self.step() >= 0 {
                    len
                } else {
                    (-len) - 1
                }
            }
        }
    }

    fn step(&self) -> i64 {
        self.step.unwrap_or(1)
    }

    fn bounds(&self, len: i64) -> (i64, i64) {
        fn normalize(i: i64, len: i64) -> i64 {
            if i >= 0 {
                i
            } else {
                len + i
            }
        }

        let step = self.step();
        let start = normalize(self.start(len), len);
        let end = normalize(self.end(len), len);

        let (lower, upper) = if step >= 0 {
            (start.max(0).min(len), end.max(0).min(len))
        } else {
            (end.max(-1).min(len - 1), start.max(-1).min(len - 1))
        };

        (lower, upper)
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

impl PathSelector for SliceSelector {
    fn select<'a>(&self, _root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        match current.as_array() {
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

impl PathVisitor for SliceSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(array) = current.as_array_mut() {
            let (lower, upper) = self.range.bounds(array.len() as i64);

            match self.range.step() {
                step @ 1..=i64::MAX => (lower..upper)
                    .step_by(step as usize)
                    .for_each(|i| visitor.visit(root, &mut array[i as usize])),
                step @ i64::MIN..=-1 => (lower + 1..=upper)
                    .rev()
                    .step_by(-step as usize)
                    .for_each(|i| visitor.visit(root, &mut array[i as usize])),
                0 => (),
            }
        }
    }
}

#[derive(Clone)]
pub struct DescendantSelector {
    selector: Box<JsonPath>,
}

impl DescendantSelector {
    pub(crate) fn new(selector: JsonPath) -> Self {
        DescendantSelector {
            selector: Box::new(selector),
        }
    }
}

impl PathSelector for DescendantSelector {
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        let mut values = self.selector.select(root, current);

        match current {
            Value::Array(array) => {
                let mut children = array
                    .iter()
                    .flat_map(|value| self.selector.select(root, value))
                    .collect();
                values.append(&mut children);
                values
            }
            Value::Object(object) => {
                let mut children = object
                    .values()
                    .flat_map(|value| self.selector.select(root, value))
                    .collect();
                values.append(&mut children);
                values
            }
            _ => values,
        }
    }
}

impl PathVisitor for DescendantSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        self.selector.visit(root, current, visitor);

        match current {
            Value::Array(array) => array
                .iter_mut()
                .for_each(|value| visitor.visit(root, value)),
            Value::Object(object) => object
                .values_mut()
                .for_each(|value| visitor.visit(root, value)),
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct FilterSelector {
    filter: Box<Filter>,
}

impl FilterSelector {
    pub(crate) fn new(filter: Filter) -> Self {
        FilterSelector {
            filter: Box::new(filter),
        }
    }
}

impl PathSelector for FilterSelector {
    fn select<'a>(&self, root: &'a Value, current: &'a Value) -> Vec<&'a Value> {
        match current {
            Value::Array(array) => array
                .iter()
                .filter(|value| self.filter.matches(root, value))
                .collect(),
            Value::Object(object) => object
                .values()
                .filter(|value| self.filter.matches(root, value))
                .collect(),
            _ => vec![],
        }
    }
}

impl PathVisitor for FilterSelector {
    fn visit<'a, F>(&self, root: &mut Value, current: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        let filter_root = root.clone();

        match current {
            Value::Array(array) => array
                .iter_mut()
                .filter(|value| self.filter.matches(&filter_root, value))
                .for_each(|value| visitor.visit(root, value)),
            Value::Object(object) => object
                .values_mut()
                .filter(|value| self.filter.matches(&filter_root, value))
                .for_each(|value| visitor.visit(root, value)),
            _ => (),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[track_caller]
    fn assert_selects<T>(selector: T, root: &Value, expected: Vec<&Value>)
    where
        T: PathSelector,
    {
        assert_eq!(selector.select(root, root), expected);
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
