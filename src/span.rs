pub type Spanned<T> = (T, Span);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span(pub std::range::Range<usize>);

impl<T> From<T> for Span
where
    T: Into<std::range::Range<usize>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl From<Span> for miette::SourceSpan {
    fn from(value: Span) -> Self {
        let range: std::ops::Range<usize> = value.0.into();
        range.into()
    }
}
