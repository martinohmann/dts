//! Provides a path compiler and the `Path` type.

mod compile;
pub mod filter;
pub mod selector;

pub use self::compile::compile;

use self::filter::FilterExpr;
use self::selector::{Select, Selector, Visit};
use crate::Error;
use dts_json::Value;
use std::slice::Iter;
use std::str::FromStr;

/// Represents a jsonpath consisting of multiple `Selector`s.
#[derive(Clone)]
pub struct Path<'a> {
    selectors: Vec<Selector<'a>>,
}

impl<'a> Path<'a> {
    /// Creates a new `Visitor` from a chain of `Selector`s.
    pub fn new<I>(selectors: I) -> Self
    where
        I: IntoIterator<Item = Selector<'a>>,
    {
        Path {
            selectors: selectors.into_iter().collect(),
        }
    }

    /// Returns an `Iterator` over all path selectors.
    pub fn iter(&self) -> Iter<'_, Selector<'a>> {
        self.selectors.iter()
    }

    /// Consumes `self` and returns the inner `Vec<Selector>`.
    pub fn into_inner(self) -> Vec<Selector<'a>> {
        self.selectors
    }

    /// Applies the path selectors to `Value` and returns references to all matches.
    pub fn select(&self, value: &'a Value) -> Vec<&'a Value> {
        self.selectors.iter().fold(vec![value], |acc, selector| {
            acc.iter()
                .flat_map(|value| selector.select(value))
                .collect()
        })
    }

    /// Recursively visits `Value` and calls `f` on every matched `Value`.
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

/// The json `Value` visitor.
///
/// An instance of this type is passed into the `visit` method of a `Selector` and recursively
/// visits every `Value` matched by all `Selector`s in the chain, calling a closure to mutate these
/// values.
pub struct Visitor<'a, F> {
    selectors: Vec<Selector<'a>>,
    mutate: &'a mut F,
}

impl<'a, F> Visitor<'a, F>
where
    F: FnMut(&mut Value),
{
    /// Creates a new `Visitor` with a chain of pending `Selector`s to apply and a mutation
    /// function that is called for every matched `Value`.
    pub fn new<I>(selectors: I, mutate: &'a mut F) -> Self
    where
        I: IntoIterator<Item = &'a Selector<'a>>,
    {
        Visitor {
            selectors: selectors.into_iter().cloned().collect(),
            mutate,
        }
    }

    /// Recursively visits `Value`.
    pub fn visit(&mut self, value: &mut Value) {
        match self.selectors.get(0) {
            Some(path) => path.visit(
                value,
                &mut Visitor::new(self.selectors.iter().skip(1), self.mutate),
            ),
            None => (self.mutate)(value),
        }
    }
}

/// Represents a comparison operation inside a filter expression.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CompOp {
    Eq,
    NotEq,
    LessEq,
    Less,
    GreaterEq,
    Greater,
    In,
}

impl FromStr for CompOp {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(CompOp::Eq),
            "!=" => Ok(CompOp::NotEq),
            "<=" => Ok(CompOp::LessEq),
            "<" => Ok(CompOp::Less),
            ">=" => Ok(CompOp::GreaterEq),
            ">" => Ok(CompOp::Greater),
            "in" => Ok(CompOp::In),
            other => Err(Error::new(format!(
                "not a comparision operation: {}",
                other
            ))),
        }
    }
}

/// Represents an array index which can be either positive or negative. Negative indices select
/// elements starting from the end of the array.
#[derive(Clone)]
pub struct Index {
    index: i64,
}

impl Index {
    pub(crate) fn new(index: i64) -> Self {
        Index { index }
    }

    pub(crate) fn get(&self, len: i64) -> Option<usize> {
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

/// Represents a slice range of the form `[start:end:step]` used in a `Slice` selector where all
/// parts can be optional signed integers.
#[derive(Default, Clone)]
pub struct SliceRange {
    start: Option<i64>,
    end: Option<i64>,
    step: Option<i64>,
}

impl SliceRange {
    pub(crate) fn new(start: Option<i64>, end: Option<i64>, step: Option<i64>) -> Self {
        SliceRange { start, end, step }
    }

    pub(crate) fn start(&self, len: i64) -> i64 {
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

    pub(crate) fn end(&self, len: i64) -> i64 {
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

    pub(crate) fn step(&self) -> i64 {
        self.step.unwrap_or(1)
    }

    pub(crate) fn bounds(&self, len: i64) -> (i64, i64) {
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
