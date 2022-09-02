/// # 5.2.3.1 Single Root Field
///
/// ## Formal Specification
/// - For each subscription operation definition `subscription` in the document
/// - Let `subscriptionType` be the root Subscription type in `schema`.
/// - Let `selectionSet` be the top level selection set on `subscription`.
/// - Let `variableValues` be the empty set.
/// - Let `groupedFieldSet` be the result of
///   `CollectFields(subscriptionType, selectionSet, variableValues)`.
/// - `groupedFieldSet` must have exactly one entry, which must not be an
///   introspection field.
pub struct SingleRootField;
