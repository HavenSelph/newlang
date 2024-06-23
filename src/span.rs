#[derive(Clone, Debug)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub filename: std::sync::Arc<str>
}

impl ariadne::Span for Span {
    type SourceId = std::sync::Arc<str>;

    fn source(&self) -> &Self::SourceId { &self.filename }

    fn start(&self) -> usize { self.start }

    fn end(&self) -> usize { self.end+1 }
}

impl Span {

    pub fn new(start: usize, end: usize, filename: std::sync::Arc<str>) -> Self {
        Span {
            start,
            end,
            filename
        }
    }

    pub fn location(index: usize, filename: std::sync::Arc<str>) -> Self {
        Self::new(index, index, filename)
    }
    pub fn extend(self, other: Span) -> Self {
        Span {
            start: self.start,
            end: other.end,
            filename: self.filename
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}:{}:{}", self.filename, self.start, self.end)
    }
}
