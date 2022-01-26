use super::filter::Filter;
use dts_json::Value;
use std::collections::VecDeque;

pub struct PathPointer<'a> {
    pub root: &'a Value,
    pub current: &'a Value,
}

impl<'a> PathPointer<'a> {
    pub(crate) fn new(root: &'a Value, current: &'a Value) -> Self {
        PathPointer { root, current }
    }
}

pub struct PathPointerMut<'a> {
    pub root: &'a mut Value,
    pub current: &'a mut Value,
}

impl<'a> PathPointerMut<'a> {
    pub(crate) fn new(root: &'a mut Value, current: &'a mut Value) -> Self {
        PathPointerMut { root, current }
    }
}

pub trait PathSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value>;
}

pub trait PathVisitor {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
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

    pub(crate) fn visit<'v>(&mut self, pointer: &mut PathPointerMut<'v>) {
        match self.chain.get(0) {
            Some(path) => {
                let mut visitor = Visitor::new(&self.chain[1..], self.mutate);
                path.visit(pointer, &mut visitor);
            }
            None => (self.mutate)(pointer.current),
        }
    }
}

impl<T> PathVisitor for Box<T>
where
    T: PathVisitor + ?Sized,
{
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        (**self).visit(pointer, visitor)
    }
}

impl<T> PathVisitor for &T
where
    T: PathVisitor + ?Sized,
{
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        (*self).visit(pointer, visitor)
    }
}

impl<T> PathSelector for Box<T>
where
    T: PathSelector + ?Sized,
{
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        (**self).select(pointer)
    }
}

impl<T> PathSelector for &T
where
    T: PathSelector + ?Sized,
{
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        (*self).select(pointer)
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        match self {
            JsonPath::Root(s) => s.select(pointer),
            JsonPath::Current(s) => s.select(pointer),
            JsonPath::Key(s) => s.select(pointer),
            JsonPath::Wildcard(s) => s.select(pointer),
            JsonPath::Index(s) => s.select(pointer),
            JsonPath::Union(s) => s.select(pointer),
            JsonPath::Slice(s) => s.select(pointer),
            JsonPath::Descendant(s) => s.select(pointer),
            JsonPath::Filter(s) => s.select(pointer),
            JsonPath::Chain(s) => s.select(pointer),
        }
    }
}

impl PathVisitor for JsonPath {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match self {
            JsonPath::Root(v) => v.visit(pointer, visitor),
            JsonPath::Current(v) => v.visit(pointer, visitor),
            JsonPath::Key(v) => v.visit(pointer, visitor),
            JsonPath::Wildcard(v) => v.visit(pointer, visitor),
            JsonPath::Index(v) => v.visit(pointer, visitor),
            JsonPath::Union(v) => v.visit(pointer, visitor),
            JsonPath::Slice(v) => v.visit(pointer, visitor),
            JsonPath::Descendant(v) => v.visit(pointer, visitor),
            JsonPath::Filter(v) => v.visit(pointer, visitor),
            JsonPath::Chain(v) => v.visit(pointer, visitor),
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        self.chain
            .iter()
            .fold(vec![pointer.current], |acc, selector| {
                acc.iter()
                    .flat_map(|value| {
                        let pointer = PathPointer::new(pointer.root, value);
                        selector.select(&pointer)
                    })
                    .collect()
            })
    }
}

impl PathVisitor for ChainSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        for path in self.chain.iter().rev() {
            visitor.chain.insert(0, path.clone());
        }

        visitor.visit(pointer);
    }
}

#[derive(Clone)]
pub struct RootSelector;

impl PathSelector for RootSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        vec![pointer.root]
    }
}

impl PathVisitor for RootSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        visitor.visit(pointer);
    }
}

#[derive(Clone)]
pub struct CurrentSelector;

impl PathSelector for CurrentSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        vec![pointer.current]
    }
}

impl PathVisitor for CurrentSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        visitor.visit(pointer);
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        pointer
            .current
            .as_object()
            .and_then(|object| object.get(&self.key))
            .map(|value| vec![value])
            .unwrap_or_default()
    }
}

impl PathVisitor for KeySelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(object) = pointer.current.as_object_mut() {
            if let Some(value) = object.get_mut(&self.key) {
                let mut pointer = PathPointerMut::new(pointer.root, value);
                visitor.visit(&mut pointer);
            }
        }
    }
}

#[derive(Clone)]
pub struct WildcardSelector;

impl PathSelector for WildcardSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        match pointer.current {
            Value::Array(array) => array.iter().collect(),
            Value::Object(object) => object.values().collect(),
            _ => vec![],
        }
    }
}

impl PathVisitor for WildcardSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match pointer.current {
            Value::Array(array) => array.iter_mut().for_each(|value| {
                let mut pointer = PathPointerMut::new(pointer.root, value);
                visitor.visit(&mut pointer);
            }),
            Value::Object(object) => object.values_mut().for_each(|value| {
                let mut pointer = PathPointerMut::new(pointer.root, value);
                visitor.visit(&mut pointer);
            }),
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        pointer
            .current
            .as_array()
            .and_then(|array| {
                self.index(array.len() as i64)
                    .map(|index| vec![&array[index]])
            })
            .unwrap_or_default()
    }
}

impl PathVisitor for IndexSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(array) = pointer.current.as_array_mut() {
            if let Some(index) = self.index(array.len() as i64) {
                let mut pointer = PathPointerMut::new(pointer.root, &mut array[index]);
                visitor.visit(&mut pointer);
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        self.entries
            .iter()
            .flat_map(|selector| selector.select(pointer))
            .collect()
    }
}

impl PathVisitor for UnionSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        for entry in self.entries.iter() {
            entry.visit(pointer, visitor)
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        match pointer.current.as_array() {
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
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        if let Some(array) = pointer.current.as_array_mut() {
            let (lower, upper) = self.range.bounds(array.len() as i64);

            match self.range.step() {
                step @ 1..=i64::MAX => (lower..upper).step_by(step as usize).for_each(|i| {
                    let mut pointer = PathPointerMut::new(pointer.root, &mut array[i as usize]);
                    visitor.visit(&mut pointer);
                }),
                step @ i64::MIN..=-1 => {
                    (lower + 1..=upper)
                        .rev()
                        .step_by(-step as usize)
                        .for_each(|i| {
                            let mut pointer =
                                PathPointerMut::new(pointer.root, &mut array[i as usize]);
                            visitor.visit(&mut pointer);
                        })
                }
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        let mut values = self.selector.select(pointer);

        match &pointer.current {
            Value::Array(array) => {
                let mut children = array
                    .iter()
                    .flat_map(|value| {
                        let pointer = PathPointer::new(pointer.root, value);
                        self.selector.select(&pointer)
                    })
                    .collect();
                values.append(&mut children);
                values
            }
            Value::Object(object) => {
                let mut children = object
                    .values()
                    .flat_map(|value| {
                        let pointer = PathPointer::new(pointer.root, value);
                        self.selector.select(&pointer)
                    })
                    .collect();
                values.append(&mut children);
                values
            }
            _ => values,
        }
    }
}

impl PathVisitor for DescendantSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        self.selector.visit(pointer, visitor);

        match pointer.current {
            Value::Array(array) => array.iter_mut().for_each(|value| {
                let mut pointer = PathPointerMut::new(pointer.root, value);
                visitor.visit(&mut pointer);
            }),
            Value::Object(object) => object.values_mut().for_each(|value| {
                let mut pointer = PathPointerMut::new(pointer.root, value);
                visitor.visit(&mut pointer);
            }),
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
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        match pointer.current {
            Value::Array(array) => array
                .iter()
                .filter(|value| {
                    let pointer = PathPointer::new(pointer.root, value);
                    self.filter.matches(&pointer)
                })
                .collect(),
            Value::Object(object) => object
                .values()
                .filter(|value| {
                    let pointer = PathPointer::new(pointer.root, value);
                    self.filter.matches(&pointer)
                })
                .collect(),
            _ => vec![],
        }
    }
}

impl PathVisitor for FilterSelector {
    fn visit<'a, 'v, F>(&self, pointer: &mut PathPointerMut<'v>, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        let root = pointer.root.clone();

        match pointer.current {
            Value::Array(array) => array
                .iter_mut()
                .filter(|value| {
                    let pointer = PathPointer::new(&root, value);
                    self.filter.matches(&pointer)
                })
                .for_each(|value| {
                    let mut pointer = PathPointerMut::new(pointer.root, value);
                    visitor.visit(&mut pointer);
                }),
            Value::Object(object) => object
                .values_mut()
                .filter(|value| {
                    let pointer = PathPointer::new(&root, value);
                    self.filter.matches(&pointer)
                })
                .for_each(|value| {
                    let mut pointer = PathPointerMut::new(pointer.root, value);
                    visitor.visit(&mut pointer);
                }),
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
        let pointer = PathPointer::new(root, root);
        assert_eq!(selector.select(&pointer), expected);
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
