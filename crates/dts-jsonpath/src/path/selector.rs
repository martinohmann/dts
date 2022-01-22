use super::filter::Filter;
use dts_json::Value;

pub struct PathPointer<'a> {
    pub root: &'a Value,
    pub current: &'a Value,
}

impl<'a> PathPointer<'a> {
    pub(crate) fn new(root: &'a Value, current: &'a Value) -> Self {
        PathPointer { root, current }
    }

    pub(crate) fn new_root(root: &'a Value) -> Self {
        PathPointer::new(root, root)
    }
}

pub trait PathSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value>;
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

pub struct JsonPath {
    chain: Vec<Box<dyn PathSelector>>,
}

impl JsonPath {
    pub(crate) fn new<I>(chain: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn PathSelector>>,
    {
        JsonPath {
            chain: chain.into_iter().collect(),
        }
    }
}

impl FromIterator<Box<dyn PathSelector>> for JsonPath {
    fn from_iter<I: IntoIterator<Item = Box<dyn PathSelector>>>(iter: I) -> Self {
        JsonPath::new(iter)
    }
}

impl<'a> IntoIterator for JsonPath {
    type Item = Box<dyn PathSelector>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chain.into_iter()
    }
}

impl PathSelector for JsonPath {
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

pub struct RootSelector;

impl PathSelector for RootSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        vec![pointer.root]
    }
}

pub struct CurrentSelector;

impl PathSelector for CurrentSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        vec![pointer.current]
    }
}

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

pub struct UnionSelector {
    entries: Vec<Box<dyn PathSelector>>,
}

impl UnionSelector {
    pub(crate) fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn PathSelector>>,
    {
        UnionSelector {
            entries: entries.into_iter().collect(),
        }
    }
}

impl FromIterator<Box<dyn PathSelector>> for UnionSelector {
    fn from_iter<I: IntoIterator<Item = Box<dyn PathSelector>>>(iter: I) -> Self {
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

#[derive(Default)]
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

pub struct SliceSelector {
    range: SliceRange,
}

impl SliceSelector {
    pub(crate) fn new(range: SliceRange) -> Self {
        SliceSelector { range }
    }

    fn slice<'a>(&self, array: &'a [Value]) -> Vec<&'a Value> {
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
}

impl PathSelector for SliceSelector {
    fn select<'a>(&self, pointer: &PathPointer<'a>) -> Vec<&'a Value> {
        match pointer.current.as_array() {
            Some(array) => self.slice(array),
            None => vec![],
        }
    }
}

pub struct DescendantSelector {
    selector: Box<dyn PathSelector>,
}

impl DescendantSelector {
    pub(crate) fn new(selector: Box<dyn PathSelector>) -> Self {
        DescendantSelector { selector }
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

pub struct FilterSelector {
    filter: Filter,
}

impl FilterSelector {
    pub(crate) fn new(filter: Filter) -> Self {
        FilterSelector { filter }
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

#[cfg(test)]
mod test {
    use super::*;
    use dts_json::json;

    #[track_caller]
    fn assert_selects<T>(selector: T, root: &Value, expected: Vec<&Value>)
    where
        T: PathSelector,
    {
        let pointer = PathPointer::new_root(root);
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
