use primes::{PrimeSet, Sieve};
use std::{
    cmp::{max, min},
    sync::{Mutex, OnceLock},
    usize,
};

use miette::SourceSpan;

fn get_primes_sieve() -> &'static Mutex<Sieve> {
    static INSTANCE: OnceLock<Mutex<Sieve>> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let mut p = Sieve::new();
        Mutex::new(p)
    })
}

/// A span of text in the source code of the program. `Span`s are asserted to run from left to right.
#[derive(Clone, Copy, Ord, PartialOrd)]
pub struct Span {
    id: u64,
    span: (usize, usize),
}

pub trait Spans {
    fn span(&self) -> Span;
}

impl Spans for Span {
    fn span(&self) -> Span {
        *self
    }
}

impl<T> Spans for &T
where
    T: Spans,
{
    fn span(&self) -> Span {
        (*self).span()
    }
}

impl Spans for (usize, usize) {
    fn span(&self) -> Span {
        Span::new(*self)
    }
}

impl Span {
    pub fn new(span: (usize, usize)) -> Self {
        assert!(span.0 <= span.1);
        Self {
            id: get_primes_sieve()
                .lock()
                .unwrap()
                .generator()
                .next()
                .unwrap(),
            span,
        }
    }

    pub fn with_id(span: (usize, usize), id: u64) -> Self {
        assert!(span.0 <= span.1);
        Self { id, span }
    }

    /// Join two `Span`s, leaving the spans unchanged, and return a new `Span` containing them.
    pub fn superspan(a: impl Spans, b: impl Spans) -> Self {
        Span::with_id(
            join_spans(a.span().span, b.span().span),
            a.span().id * b.span().id,
        )
    }

    pub fn together<const N: usize>(spans: [impl Spans; N]) -> Self {
        assert!(N > 0);
        let low_bound: usize = spans
            .iter()
            .fold(spans[0].span().span.0, |x, y| min(x, y.span().span.0));
        let high_bound: usize = spans
            .iter()
            .fold(spans[0].span().span.1, |x, y| max(x, y.span().span.1));
        let id: u64 = spans.iter().map(|x| x.span().id).product();

        Self {
            span: (low_bound, high_bound),
            id,
        }
    }

    /// Mutably extend the bounds of the span, to contain `other`.
    pub fn join(&mut self, other: impl Spans) {
        self.span = join_spans(self.span, other.span().span)
    }

    /// Returns weather `other` is disjoint from `self`, i.e. if their spans have no overlap.
    pub fn is_disjoint(&self, other: &Span) -> bool {
        self.span.1 <= other.span.0 || other.span.1 <= self.span.0
    }

    /// Returns weather `other` overlaps with `self`.
    pub fn overlaps(&self, other: &Span) -> bool {
        !self.is_disjoint(other)
    }

    /// Returns weather `self` is a superset of `other`, i.e. `other` is fully contained within `self`
    pub fn is_superset(&self, other: &Span) -> bool {
        self.span.0 <= other.span.0 && other.span.1 <= self.span.1
    }

    pub fn span(&self) -> (usize, usize) {
        self.span
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the id's of Spans this Span was generated from, based on the prime factors of the Spans id.
    pub fn composing_ids(&self) -> Vec<u64> {
        primes::factors_uniq(self.id)
    }
}

fn join_spans(a: (usize, usize), b: (usize, usize)) -> (usize, usize) {
    (min(a.0, b.0), max(a.1, b.1))
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.span.0, self.span.1)
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.span.0, self.span.1)
    }
}

impl std::cmp::Eq for Span {}

impl std::cmp::PartialEq for Span {
    fn eq(&self, other: &Self) -> bool {
        self.span.0 == other.span.0 && self.span.1 == other.span.1
    }

    fn ne(&self, other: &Self) -> bool {
        self.span.0 != other.span.0 || self.span.1 != other.span.1
    }
}

impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        SourceSpan::from((self.span.0, self.span.1 - self.span.0))
    }
}

#[cfg(test)]
mod tests {
    use super::Span;

    #[test]
    fn span_is_subset_of_self() {
        let span = Span::new((0, 5));
        assert!(span.is_superset(&span))
    }

    #[test]
    fn span_is_not_disjoint_with_self() {
        let span = Span::new((0, 5));
        assert!(!span.is_disjoint(&span))
    }
}
