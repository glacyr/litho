use wrom::{alt, many, Input, RecoverableParser};
use wrom_derive::wrom;

use crate::ast::*;
use crate::lex::Token;

mod combinators;
pub mod executable;
mod parse;
mod recovery;
pub mod schema;
mod stream;

pub use parse::Parse;
pub use recovery::RecoveryPoint;
pub use stream::Stream;

const RECURSION_LIMIT: usize = 32;

impl<I> wrom::Missing<I> for Missing
where
    I: Spanned,
{
    type Error = MissingToken;

    #[inline]
    fn error(&self, input: &mut I) -> Self::Error {
        MissingToken {
            span: input.span(),
            missing: *self,
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Incomplete,
    Expected(RecoveryPoint),
    MaxRecursion(Span),
}

impl Error {
    pub fn max_recursion<I>(input: &mut I) -> Self
    where
        I: Spanned,
    {
        Error::MaxRecursion(input.span())
    }
}

#[wrom]
pub fn document<'a, T, I>() -> impl RecoverableParser<I, Document<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: Clone + 'a,
{
    many(definition().map(Into::into)).map(|definitions| Document { definitions })
}

#[wrom]
pub fn definition<'a, T, I>() -> impl RecoverableParser<I, Definition<T>, Error, RecoveryPoint> + 'a
where
    I: Input<Item = Token<T>> + Spanned + 'a,
    T: Clone + 'a,
{
    alt((
        executable::executable_definition().map(Definition::ExecutableDefinition),
        schema::type_system_definition_or_extension()
            .map(Definition::TypeSystemDefinitionOrExtension),
    ))
}

macro_rules! parse {
    (Arc<$name:ident>, $($fn:tt)*) => {
        impl<T> Parse<T> for Arc<$name<T>> {
            fn parse<'a>(stream: Stream<'a, T>) -> Result<(Self, Vec<Token<T>>), Err<Error>>
            where
                T: From<&'a str> + Clone,
            {
                $($fn)*
                    .parse(stream, Default::default())
                    .map(|(input, value)| (value, input.into_unexpected()))
            }
        }
    };

    ($name:ident, $($fn:tt)*) => {
        impl<T> Parse<T> for $name<T> {
            fn parse<'a>(mut stream: Stream<'a, T>) -> Result<(Self, Vec<Token<T>>), Error>
            where
                T: From<&'a str> + Clone,
            {
                let value = $($fn)*
                    .parse(&mut stream, Default::default())?;

                Ok((value, stream.into_unexpected()))
            }
        }
    };
}

parse!(Document, document());
// parse!(Definition, definition);
parse!(ExecutableDocument, executable::executable_document());
// parse!(ExecutableDefinition, executable::executable_definition);
// parse!(OperationDefinition, executable::operation_definition());
// parse!(SchemaDefinition, schema::schema_definition());
// parse!(ExecutableDocument, executable::executable_document());
parse!(TypeSystemDocument, schema::type_system_document());
// parse!(DirectiveDefinition, schema::directive_definition());
// parse!(OperationType, executable::operation_type);
// parse!(SelectionSet, executable::selection_set);
// parse!(Selection, executable::selection);
// parse!(Field, executable::field);
// parse!(Alias, executable::alias);
// parse!(Arguments, executable::arguments);
// parse!(Argument, executable::argument);
// parse!(FragmentSpread, executable::fragment_spread);
// parse!(InlineFragment, executable::inline_fragment);
// parse!(FragmentDefinition, executable::fragment_definition);
// parse!(TypeCondition, executable::type_condition);
// parse!(Arc<Value>, executable::value(RECURSION_LIMIT));
// parse!(BooleanValue, executable::boolean_value);
// parse!(NullValue, executable::null_value);
// parse!(EnumValue, executable::enum_value);
// parse!(ListValue, executable::list_value);
// parse!(ObjectValue, executable::object_value);
// parse!(VariableDefinitions, executable::variable_definitions());
// parse!(Arc<VariableDefinition>, executable::variable_definition());
// parse!(Variable, executable::variable);
// parse!(Arc<Type>, executable::ty(RECURSION_LIMIT));
// parse!(NamedType, executable::named_type);
// parse!(NonNullType, executable::non_null_type);
// parse!(Directives, executable::directives);
// parse!(Directive, executable::directive);
