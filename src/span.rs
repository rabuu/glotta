use std::fmt;

#[derive(Clone, Copy, PartialEq)]
pub struct Span {
    pub inner: std::range::Range<usize>,
}

impl Span {
    pub fn to(self, to: Span) -> Span {
        (self.inner.start..to.inner.end).into()
    }
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> From<T> for Span
where
    T: Into<std::range::Range<usize>>,
{
    fn from(value: T) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(value: Span) -> Self {
        let range: std::ops::Range<usize> = value.inner.into();
        range.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourcePosition {
    pub row: usize,
    pub col: usize,
}

impl fmt::Display for SourcePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let SourcePosition { row, col } = self;
        write!(f, "{row}:{col}")
    }
}
