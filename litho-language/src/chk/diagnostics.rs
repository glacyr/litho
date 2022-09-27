use std::fmt::Display;

use crate::lex::Span;

pub trait LabelBuilder {
    type Label;

    fn new(span: Span) -> Self;

    fn with_message<M>(self, msg: M) -> Self
    where
        M: ToString;

    fn with_color(self, color: ariadne::Color) -> Self;

    fn finish(self) -> Self::Label;
}

pub trait IntoReport {
    fn into_report<B>(self) -> B::Report
    where
        B: ReportBuilder;
}

pub trait ReportBuilder {
    type LabelBuilder: LabelBuilder;
    type Report;

    fn new(kind: ariadne::ReportKind, source_id: usize, offset: usize) -> Self;

    fn with_code<C>(self, code: C) -> Self
    where
        C: Display;

    fn with_message<M>(self, msg: M) -> Self
    where
        M: ToString;

    fn with_help<N>(self, note: N) -> Self
    where
        N: ToString;

    fn with_label(self, label: Self::LabelBuilder) -> Self;

    fn finish(self) -> Self::Report;
}

impl LabelBuilder for ariadne::Label<Span> {
    type Label = Self;

    fn new(span: Span) -> Self {
        Self::new(span)
    }

    fn with_message<M>(self, msg: M) -> Self
    where
        M: ToString,
    {
        self.with_message(msg)
    }

    fn with_color(self, color: ariadne::Color) -> Self {
        self.with_color(color)
    }

    fn finish(self) -> Self {
        self
    }
}

impl ReportBuilder for ariadne::ReportBuilder<Span> {
    type LabelBuilder = ariadne::Label<Span>;
    type Report = ariadne::Report<Span>;

    fn new(kind: ariadne::ReportKind, source_id: usize, offset: usize) -> Self {
        ariadne::Report::build(kind, source_id, offset)
    }

    fn with_code<C>(self, code: C) -> Self
    where
        C: Display,
    {
        self.with_code(code)
    }

    fn with_message<M>(self, msg: M) -> Self
    where
        M: ToString,
    {
        self.with_message(msg)
    }

    fn with_help<N>(self, note: N) -> Self
    where
        N: ToString,
    {
        self.with_help(note)
    }

    fn with_label(self, label: Self::LabelBuilder) -> Self {
        self.with_label(label)
    }

    fn finish(self) -> Self::Report {
        self.finish()
    }
}
