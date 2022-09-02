mod traverse;
mod visitor;

pub use traverse::Traverse;
pub use visitor::Visitor;

use crate::extensions::Span;

#[derive(Clone, Copy)]
pub enum Scope<'a, 'b> {
    Document,

    Query {
        up: &'a Scope<'a, 'b>,
        span: Span,
    },

    Mutation {
        up: &'a Scope<'a, 'b>,
        span: Span,
    },

    Subscription {
        up: &'a Scope<'a, 'b>,
        span: Span,
    },

    InlineFragment {
        up: &'a Scope<'a, 'b>,
        ty: &'b str,
        span: Span,
    },

    Fragment {
        up: &'a Scope<'a, 'b>,
        name: &'b str,
        ty: &'b str,
        span: Span,
    },

    Field {
        up: &'a Scope<'a, 'b>,
        name: &'b str,
        span: Span,
        ty: &'b str,
    },
}

impl<'a, 'b> Scope<'a, 'b> {
    pub fn query(&'a self, span: Span) -> Scope<'a, 'b> {
        Scope::Query { up: self, span }
    }

    pub fn mutation(&'a self, span: Span) -> Scope<'a, 'b> {
        Scope::Mutation { up: self, span }
    }

    pub fn subscription(&'a self, span: Span) -> Scope<'a, 'b> {
        Scope::Subscription { up: self, span }
    }

    pub fn inline_fragment(&'a self, ty: &'b str, span: Span) -> Scope<'a, 'b> {
        Scope::InlineFragment { up: self, ty, span }
    }

    pub fn fragment(&'a self, name: &'b str, ty: &'b str, span: Span) -> Scope<'a, 'b> {
        Scope::Fragment {
            up: self,
            name,
            ty,
            span,
        }
    }

    pub fn field(&'a self, name: &'b str, span: Span, ty: &'b str) -> Scope<'a, 'b> {
        Scope::Field {
            up: self,
            name,
            span,
            ty,
        }
    }

    pub fn up(&self) -> Option<&Scope<'a, 'b>> {
        match self {
            Self::Document => None,
            Self::Query { up, .. }
            | Self::Mutation { up, .. }
            | Self::Subscription { up, .. }
            | Self::InlineFragment { up, .. }
            | Self::Fragment { up, .. }
            | Self::Field { up, .. } => Some(up),
        }
    }

    pub fn ty(&self) -> &'b str {
        match self {
            Self::Document | Self::Query { .. } => "Query",
            Self::Mutation { .. } => "Mutation",
            Self::Subscription { .. } => "Subscription",
            Self::InlineFragment { ty, .. } => ty,
            Self::Fragment { ty, .. } => ty,
            Self::Field { ty, .. } => ty,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Self::Document => Default::default(),
            Self::Query { span, .. }
            | Self::Mutation { span, .. }
            | Self::Subscription { span, .. }
            | Self::InlineFragment { span, .. }
            | Self::Fragment { span, .. }
            | Self::Field { span, .. } => span.clone(),
        }
    }

    pub fn field_name(&self) -> Option<&'b str> {
        match self {
            Self::Field { name, .. } => Some(name),
            _ => self.up().and_then(|scope| scope.field_name()),
        }
    }

    pub fn is_fragment(&self) -> bool {
        match self {
            Self::Fragment { .. } | Self::InlineFragment { .. } => true,
            _ => false,
        }
    }
}
