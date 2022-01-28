use super::{FilterExpr, Index, SliceRange};
use dts_json::Value;

pub trait Select<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value>;
}

pub trait Visit<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value);
}

pub struct Visitor<'a, F> {
    selectors: Vec<Selector<'a>>,
    mutate: &'a mut F,
}

impl<'a, F> Visitor<'a, F>
where
    F: FnMut(&mut Value),
{
    pub fn new<I>(selectors: I, mutate: &'a mut F) -> Self
    where
        I: IntoIterator<Item = &'a Selector<'a>>,
    {
        Visitor {
            selectors: selectors.into_iter().cloned().collect(),
            mutate,
        }
    }

    pub fn visit(&mut self, value: &mut Value) {
        match self.selectors.get(0) {
            Some(path) => path.visit(value, &mut Visitor::new(&self.selectors[1..], self.mutate)),
            None => (self.mutate)(value),
        }
    }
}

#[derive(Clone)]
pub enum Selector<'a> {
    Root(Root<'a>),
    Current(Current),
    Key(ObjectKey),
    Wildcard(Wildcard),
    Index(ArrayIndex),
    Union(Union<'a>),
    Slice(Slice),
    Descendant(Descendant<'a>),
    Filter(Filter<'a>),
}

impl<'a> Select<'a> for Selector<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        match self {
            Selector::Root(s) => s.select(value),
            Selector::Current(s) => s.select(value),
            Selector::Key(s) => s.select(value),
            Selector::Wildcard(s) => s.select(value),
            Selector::Index(s) => s.select(value),
            Selector::Union(s) => s.select(value),
            Selector::Slice(s) => s.select(value),
            Selector::Descendant(s) => s.select(value),
            Selector::Filter(s) => s.select(value),
        }
    }
}

impl<'a> Visit<'a> for Selector<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match self {
            Selector::Root(v) => v.visit(value, visitor),
            Selector::Current(v) => v.visit(value, visitor),
            Selector::Key(v) => v.visit(value, visitor),
            Selector::Wildcard(v) => v.visit(value, visitor),
            Selector::Index(v) => v.visit(value, visitor),
            Selector::Union(v) => v.visit(value, visitor),
            Selector::Slice(v) => v.visit(value, visitor),
            Selector::Descendant(v) => v.visit(value, visitor),
            Selector::Filter(v) => v.visit(value, visitor),
        }
    }
}

#[derive(Clone)]
pub struct Path<'a> {
    selectors: Vec<Selector<'a>>,
}

impl<'a> Path<'a> {
    pub(crate) fn new<I>(selectors: I) -> Self
    where
        I: IntoIterator<Item = Selector<'a>>,
    {
        Path {
            selectors: selectors.into_iter().collect(),
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Selector<'a>> {
        self.selectors.iter()
    }

    pub fn into_inner(self) -> Vec<Selector<'a>> {
        self.selectors
    }

    pub fn visit<F>(&self, value: &mut Value, mut f: F)
    where
        F: FnMut(&mut Value),
    {
        let mut visitor = Visitor::new(self.iter(), &mut f);
        visitor.visit(value);
    }
}

impl<'a> FromIterator<Selector<'a>> for Path<'a> {
    fn from_iter<I: IntoIterator<Item = Selector<'a>>>(iter: I) -> Self {
        Path::new(iter)
    }
}

impl<'a> IntoIterator for Path<'a> {
    type Item = Selector<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.selectors.into_iter()
    }
}

impl<'a> From<Selector<'a>> for Path<'a> {
    fn from(selector: Selector<'a>) -> Self {
        Path::from_iter(vec![selector])
    }
}

impl<'a> Select<'a> for Path<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        self.selectors.iter().fold(vec![value], |acc, selector| {
            acc.iter()
                .flat_map(|value| selector.select(value))
                .collect()
        })
    }
}

#[derive(Clone)]
pub struct Root<'a> {
    root: &'a Value,
}

impl<'a> Root<'a> {
    pub(crate) fn new(root: &'a Value) -> Self {
        Root { root }
    }
}

impl<'a> Select<'a> for Root<'a> {
    fn select(&self, _value: &'a Value) -> Vec<&'a Value> {
        vec![self.root]
    }
}

impl<'a> Visit<'a> for Root<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        // This is not a bug. Root is guaranteed to be the first selector in which case the current
        // value == root. Working with a mutable reference to self.root would lead to two mutable
        // borrows of the same data, which is not allowed in safe rust.
        visitor.visit(value);
    }
}

#[derive(Clone)]
pub struct Current;

impl<'a> Select<'a> for Current {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        vec![value]
    }
}

impl<'a> Visit<'a> for Current {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        visitor.visit(value);
    }
}

#[derive(Clone)]
pub struct ObjectKey {
    key: String,
}

impl ObjectKey {
    pub(crate) fn new(key: String) -> Self {
        ObjectKey { key }
    }
}

impl<'a> Select<'a> for ObjectKey {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        value
            .as_object()
            .and_then(|object| object.get(&self.key))
            .map(|value| vec![value])
            .unwrap_or_default()
    }
}

impl<'a> Visit<'a> for ObjectKey {
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
pub struct Wildcard;

impl<'a> Select<'a> for Wildcard {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        match value {
            Value::Array(array) => array.iter().collect(),
            Value::Object(object) => object.values().collect(),
            _ => vec![],
        }
    }
}

impl<'a> Visit<'a> for Wildcard {
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
pub struct ArrayIndex {
    index: Index,
}

impl ArrayIndex {
    pub(crate) fn new(index: i64) -> Self {
        ArrayIndex {
            index: Index::new(index),
        }
    }
}

impl<'a> Select<'a> for ArrayIndex {
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

impl<'a> Visit<'a> for ArrayIndex {
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
pub struct Union<'a> {
    entries: Vec<Selector<'a>>,
}

impl<'a> Union<'a> {
    pub(crate) fn new<I>(entries: I) -> Self
    where
        I: IntoIterator<Item = Selector<'a>>,
    {
        Union {
            entries: entries.into_iter().collect(),
        }
    }
}

impl<'a> FromIterator<Selector<'a>> for Union<'a> {
    fn from_iter<I: IntoIterator<Item = Selector<'a>>>(iter: I) -> Self {
        Union::new(iter)
    }
}

impl<'a> Select<'a> for Union<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        self.entries
            .iter()
            .flat_map(|selector| selector.select(value))
            .collect()
    }
}

impl<'a> Visit<'a> for Union<'a> {
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
pub struct Slice {
    range: SliceRange,
}

impl Slice {
    pub(crate) fn new(range: SliceRange) -> Self {
        Slice { range }
    }
}

impl<'a> Select<'a> for Slice {
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

impl<'a> Visit<'a> for Slice {
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
pub struct Descendant<'a> {
    selector: Box<Selector<'a>>,
}

impl<'a> Descendant<'a> {
    pub(crate) fn new(selector: Selector<'a>) -> Self {
        Descendant {
            selector: Box::new(selector),
        }
    }
}

impl<'a> Select<'a> for Descendant<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        let mut values = self.selector.select(value);

        match value {
            Value::Array(array) => {
                let mut children = array.iter().flat_map(|value| self.select(value)).collect();
                values.append(&mut children);
                values
            }
            Value::Object(object) => {
                let mut children = object
                    .values()
                    .flat_map(|value| self.select(value))
                    .collect();
                values.append(&mut children);
                values
            }
            _ => values,
        }
    }
}

impl<'a> Visit<'a> for Descendant<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        self.selector.visit(value, visitor);

        match value {
            Value::Array(array) => array
                .iter_mut()
                .for_each(|value| self.visit(value, visitor)),
            Value::Object(object) => object
                .values_mut()
                .for_each(|value| self.visit(value, visitor)),
            _ => (),
        }
    }
}

#[derive(Clone)]
pub struct Filter<'a> {
    expr: Box<FilterExpr<'a>>,
}

impl<'a> Filter<'a> {
    pub(crate) fn new(expr: FilterExpr<'a>) -> Self {
        Filter {
            expr: Box::new(expr),
        }
    }
}

impl<'a> Select<'a> for Filter<'a> {
    fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        match value {
            Value::Array(array) => array
                .iter()
                .filter(|value| self.expr.matches(value))
                .collect(),
            Value::Object(object) => object
                .values()
                .filter(|value| self.expr.matches(value))
                .collect(),
            _ => vec![],
        }
    }
}

impl<'a> Visit<'a> for Filter<'a> {
    fn visit<F>(&self, value: &mut Value, visitor: &mut Visitor<'a, F>)
    where
        F: FnMut(&mut Value),
    {
        match value {
            Value::Array(array) => array
                .iter_mut()
                .filter(|value| self.expr.matches(value))
                .for_each(|value| visitor.visit(value)),
            Value::Object(object) => object
                .values_mut()
                .filter(|value| self.expr.matches(value))
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
        T: Select<'a>,
    {
        assert_eq!(selector.select(root), expected);
    }

    #[test]
    fn test_array_index_selector() {
        let selector = ArrayIndex::new(0);
        assert_selects(selector, &json!([1, 2]), vec![&json!(1)]);

        let selector = ArrayIndex::new(-2);
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2)]);
    }

    #[test]
    fn test_slice_selector() {
        let selector = Slice::new(SliceRange::default());
        assert_selects(
            selector,
            &json!([1, 2, 3]),
            vec![&json!(1), &json!(2), &json!(3)],
        );

        let selector = Slice::new(SliceRange::new(Some(1), None, None));
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2), &json!(3)]);

        let selector = Slice::new(SliceRange::new(Some(1), Some(3), None));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5]),
            vec![&json!(2), &json!(3)],
        );

        let selector = Slice::new(SliceRange::new(Some(1), Some(5), Some(2)));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5]),
            vec![&json!(2), &json!(4)],
        );

        let selector = Slice::new(SliceRange::new(Some(5), Some(1), Some(-2)));
        assert_selects(
            selector,
            &json!([1, 2, 3, 4, 5, 6]),
            vec![&json!(6), &json!(4)],
        );

        let selector = Slice::new(SliceRange::new(None, None, Some(-1)));
        assert_selects(
            selector,
            &json!([1, 2, 3]),
            vec![&json!(3), &json!(2), &json!(1)],
        );

        let selector = Slice::new(SliceRange::new(Some(-2), Some(-1), None));
        assert_selects(selector, &json!([1, 2, 3]), vec![&json!(2)]);

        let selector = Slice::new(SliceRange::new(Some(10), Some(12), None));
        assert_selects(selector, &json!([1, 2, 3]), vec![]);
    }

    #[test]
    fn test_object_key_selector() {
        let selector = ObjectKey::new("foo".into());
        assert_selects(selector, &json!({"foo": 1, "bar": 2}), vec![&json!(1)]);
    }
}
