pub mod driver;
pub mod parser;

pub type Span = std::range::Range<usize>;
pub type Spanned<T> = (T, Span);
