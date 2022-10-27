use super::SourceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub source_id: SourceId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn before(&self, index: usize) -> bool {
        self.end < index
    }

    pub fn contains(&self, index: usize) -> bool {
        self.start <= index && index <= self.end
    }

    pub fn join(&mut self, other: Span) {
        self.start = self.start.min(other.start);
        self.end = self.end.max(other.end);
    }

    pub fn joined(mut self, other: Span) -> Span {
        self.join(other);
        self
    }

    pub fn between(left: Self, right: Self) -> Span {
        Span {
            source_id: left.source_id,
            start: left.end,
            end: right.start,
        }
    }

    pub fn collapse_to_start(self) -> Self {
        Self {
            source_id: self.source_id,
            start: self.start,
            end: self.start,
        }
    }

    pub fn collapse_to_end(self) -> Self {
        Self {
            source_id: self.source_id,
            start: self.end,
            end: self.end,
        }
    }
}

impl ariadne::Span for Span {
    type SourceId = SourceId;

    fn source(&self) -> &Self::SourceId {
        &self.source_id
    }

    fn start(&self) -> usize {
        self.start
    }

    fn end(&self) -> usize {
        self.end
    }
}
