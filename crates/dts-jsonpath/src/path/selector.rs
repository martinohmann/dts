use super::filter::Filter;
use dts_json::Value;

pub struct Values<'a> {
    pub root: &'a Value,
    pub current: &'a Value,
}

impl<'a> Values<'a> {
    pub(crate) fn new(root: &'a Value, current: &'a Value) -> Self {
        Values { root, current }
    }

    pub(crate) fn new_root(root: &'a Value) -> Self {
        Values::new(root, root)
    }
}

pub trait Selector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value>;
}

impl<T> Selector for Box<T>
where
    T: Selector + ?Sized,
{
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        (**self).select(values)
    }
}

impl<T> Selector for &T
where
    T: Selector + ?Sized,
{
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        (*self).select(values)
    }
}

pub struct JsonPath {
    chain: Vec<Box<dyn Selector>>,
}

impl JsonPath {
    pub(crate) fn new<I>(chain: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn Selector>>,
    {
        JsonPath {
            chain: chain.into_iter().collect(),
        }
    }
}

impl FromIterator<Box<dyn Selector>> for JsonPath {
    fn from_iter<I: IntoIterator<Item = Box<dyn Selector>>>(iter: I) -> Self {
        JsonPath::new(iter)
    }
}

impl<'a> IntoIterator for JsonPath {
    type Item = Box<dyn Selector>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.chain.into_iter()
    }
}

impl Selector for JsonPath {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        self.chain
            .iter()
            .fold(vec![values.current], |acc, selector| {
                acc.iter()
                    .flat_map(|value| {
                        let values = Values::new(values.root, value);
                        selector.select(&values)
                    })
                    .collect()
            })
    }
}

pub struct RootSelector;

impl Selector for RootSelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        vec![values.root]
    }
}

pub struct CurrentSelector;

impl Selector for CurrentSelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        vec![values.current]
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

impl Selector for KeySelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        values
            .current
            .as_object()
            .and_then(|object| object.get(&self.key))
            .map(|value| vec![value])
            .unwrap_or_default()
    }
}

pub struct WildcardSelector;

impl Selector for WildcardSelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        match values.current {
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

impl Selector for IndexSelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        values
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
    entries: Vec<Box<dyn Selector>>,
}

impl UnionSelector {
    pub(crate) fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn Selector>>,
    {
        UnionSelector {
            entries: entries.into_iter().collect(),
        }
    }
}

impl FromIterator<Box<dyn Selector>> for UnionSelector {
    fn from_iter<I: IntoIterator<Item = Box<dyn Selector>>>(iter: I) -> Self {
        UnionSelector::new(iter)
    }
}

impl Selector for UnionSelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        self.entries
            .iter()
            .flat_map(|selector| selector.select(values))
            .collect()
    }
}

pub struct SliceSelector {
    start: Option<i64>,
    end: Option<i64>,
    step: Option<i64>,
}

impl SliceSelector {
    pub(crate) fn new(start: Option<i64>, end: Option<i64>, step: Option<i64>) -> Self {
        SliceSelector { start, end, step }
    }

    fn start(&self, step: i64, len: i64) -> i64 {
        match self.start {
            Some(start) => start,
            None => {
                if step >= 0 {
                    0
                } else {
                    len - 1
                }
            }
        }
    }

    fn end(&self, step: i64, len: i64) -> i64 {
        match self.end {
            Some(end) => end,
            None => {
                if step >= 0 {
                    len
                } else {
                    (-len) - 1
                }
            }
        }
    }

    fn step(&self) -> i64 {
        match self.step {
            Some(step) => step,
            None => 1,
        }
    }

    fn bounds(&self, step: i64, len: i64) -> (i64, i64) {
        fn normalize(i: i64, len: i64) -> i64 {
            if i >= 0 {
                i
            } else {
                len + i
            }
        }

        let start = normalize(self.start(step, len), len);
        let end = normalize(self.end(step, len), len);

        let (lower, upper) = if step >= 0 {
            (start.max(0).min(len), end.max(0).min(len))
        } else {
            (end.max(-1).min(len - 1), start.max(-1).min(len - 1))
        };

        (lower, upper)
    }

    fn slice_array<'a>(&self, array: &'a Vec<Value>) -> Vec<&'a Value> {
        let step = self.step();
        let (lower, upper) = self.bounds(step, array.len() as i64);

        if step > 0 {
            (lower..upper)
                .step_by(step as usize)
                .map(|i| &array[i as usize])
                .collect()
        } else if step < 0 {
            (lower + 1..=upper)
                .rev()
                .step_by(-step as usize)
                .map(|i| &array[i as usize])
                .collect()
        } else {
            vec![]
        }
    }
}

impl Selector for SliceSelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        match values.current.as_array() {
            Some(array) => self.slice_array(array),
            None => vec![],
        }
    }
}

pub struct DescendantSelector {
    _selector: Box<dyn Selector>,
}

impl DescendantSelector {
    pub(crate) fn new(selector: Box<dyn Selector>) -> Self {
        DescendantSelector {
            _selector: selector,
        }
    }
}

impl Selector for DescendantSelector {
    fn select<'a>(&self, _values: &Values<'a>) -> Vec<&'a Value> {
        unimplemented!()
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

impl Selector for FilterSelector {
    fn select<'a>(&self, values: &Values<'a>) -> Vec<&'a Value> {
        match values.current {
            Value::Array(array) => array
                .iter()
                .filter(|value| {
                    let values = Values::new(values.root, value);
                    self.filter.matches(&values)
                })
                .collect(),
            Value::Object(object) => object
                .values()
                .filter(|value| {
                    let values = Values::new(values.root, value);
                    self.filter.matches(&values)
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
        T: Selector,
    {
        let values = Values::new_root(root);
        assert_eq!(selector.select(&values), expected);
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
        let selector = SliceSelector::new(None, None, None);
        assert_selects(
            selector,
            &json!([1, 2, 3]),
            vec![&json!(1), &json!(2), &json!(3)],
        );

        let selector = SliceSelector::new(Some(1), None, None);
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2), &json!(3)]);

        let selector = SliceSelector::new(Some(1), Some(3), None);
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5]),
            vec![&json!(2), &json!(3)],
        );

        let selector = SliceSelector::new(Some(1), Some(5), Some(2));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5]),
            vec![&json!(2), &json!(4)],
        );

        let selector = SliceSelector::new(Some(5), Some(1), Some(-2));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5, 6]),
            vec![&json!(6), &json!(4)],
        );

        let selector = SliceSelector::new(None, None, Some(-1));
        assert_selects(
            selector,
            &json!([1, 2, 3]),
            vec![&json!(3), &json!(2), &json!(1)],
        );

        let selector = SliceSelector::new(Some(-2), Some(-1), None);
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2)]);

        let selector = SliceSelector::new(Some(10), Some(12), None);
        assert_selects(selector, &json!([1, 2, 3]), vec![]);
    }

    #[test]
    fn test_key_selector() {
        let selector = KeySelector::new("foo".into());
        assert_selects(selector, &json!({"foo": 1, "bar": 2}), vec![&json!(1)]);
    }
}
