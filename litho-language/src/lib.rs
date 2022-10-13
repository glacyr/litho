// #![warn(missing_docs)]

pub mod ast;
pub mod chk;
pub mod lex;
pub mod syn;

pub use ast::Document;
pub use syn::{parse_from_str, Parse};

#[doc(hidden)]
pub use ariadne;

#[cfg(test)]
mod tests {
    use ariadne::Source;

    use super::chk::{Error, Errors, IntoReport};
    use super::lex::{Span, Token};
    use super::{Document, Parse};

    #[test]
    fn it_works() {
        let source = r#"type Query {
            feedbackQuestionCategory(id: ID!): FeedbackQuestionCategory!
            feedbackQuestionCategories(isArchived: Boolean, before: ID, after: ID, first: Int, last: Int): FeedbackQuestionCategoryConnection!
        }
        
        type Mutation {
            createFeedbackQuestionCategory(input: CreateFeedbackQuestionCategoryInput!): CreateFeedbackQuestionCategoryResponse!
            updateFeedbackQuestionCategory(input: UpdateFeedbackQuestionCategoryInput!): UpdateFeedbackQuestionCategoryResponse!
            reorderFeedbackQuestionCategory(input: ReorderFeedbackQuestionCategoryInput!): ReorderFeedbackQuestionCategoryResponse!
            archiveFeedbackQuestionCategory(input: ArchiveFeedbackQuestionCategoryInput!): ArchiveFeedbackQuestionCategoryResponse!
            restoreFeedbackQuestionCategory(input: RestoreFeedbackQuestionCategoryInput!): RestoreFeedbackQuestionCategoryResponse!
        }
        
        type FeedbackQuestionCategoryConnection {
            edges: [FeedbackQuestionCategoryEdge!]!
            pageInfo: PageInfo!
        }
        
        type FeedbackQuestionCategoryEdge {
            node: FeedbackQuestionCategory!
            cursor: ID!
        }
        
        type FeedbackQuestionCategory {
            id: ID!
            name: LocalizedString!
            order: Float!
            isArchived: Boolean!
        }
        
        input CreateFeedbackQuestionCategoryInput {
            name: LocalizedString!
        }
        
        type CreateFeedbackQuestionCategoryResponse {
            feedbackQuestionCategory: FeedbackQuestionCategory!
        }
        
        input UpdateFeedbackQuestionCategoryInput {
            id: ID!
            name: LocalizedString!
        }
        
        type UpdateFeedbackQuestionCategoryResponse {
            feedbackQuestionCategory: FeedbackQuestionCategory!
        }
        
        input ReorderFeedbackQuestionCategoryInput {
            id: ID!
            order: Float!
        }
        
        type ReorderFeedbackQuestionCategoryResponse {
            feedbackQuestionCategory: FeedbackQuestionCategory!
        }
        
        input ArchiveFeedbackQuestionCategoryInput {
            id: ID!
        }
        
        type ArchiveFeedbackQuestionCategoryResponse {
            feedbackQuestionCategory: FeedbackQuestionCategory!
        }
        
        input RestoreFeedbackQuestionCategoryInput {
            id: ID!
        }
        
        type RestoreFeedbackQuestionCategoryResponse {
            feedbackQuestionCategory: FeedbackQuestionCategory!
        }"#;
        let (ast, unrecognized) = Document::parse_from_str(Default::default(), source).unwrap();

        println!("Result: {:#?} (errors: {:#?})", ast, ast.errors());

        for error in
            ast.errors()
                .into_iter()
                .chain(unrecognized.into_iter().map(|token: Token<&str>| {
                    Error::UnrecognizedTokens {
                        tokens: vec![token],
                    }
                }))
        {
            error
                .into_report::<ariadne::ReportBuilder<Span>>()
                .print((Default::default(), Source::from(source)))
                .unwrap();
        }
    }
}
