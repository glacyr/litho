use litho_language::ast::*;
use smol_str::SmolStr;
use tower_lsp::lsp_types::*;

use super::{line_col_to_offset, Document};

pub struct DefinitionProvider<'ast>(&'ast Document);

impl DefinitionProvider<'_> {
    pub fn new<'ast>(document: &'ast Document) -> DefinitionProvider<'ast> {
        DefinitionProvider(document)
    }

    pub fn goto_definition(&self, position: Position) -> Option<GotoDefinitionResponse> {
        // let index = line_col_to_offset(self.0.text(), position.line, position.character);
        // let mut hover = None;

        // self.0.ast().traverse(&DefinitionVisitor(index), &mut hover);

        // hover

        Some(GotoDefinitionResponse::Scalar(Location {
            uri: self.0.url().clone(),
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
        }))
    }
}

// struct DefinitionVisitor(usize);

// impl<'ast, 'a> Visit<'ast, 'a> for DefinitionVisitor {
//     type Accumulator = Option<Definition>;

//     fn visit_object_type_definition(
//         &self,
//         node: &'ast ObjectTypeDefinition<'a>,
//         accumulator: &mut Self::Accumulator,
//     ) {
//         if node.ty.span().joined(node.name.span()).contains(self.0) {
//             accumulator.replace(Definition {
//                 contents: DefinitionContents::Scalar(MarkedString::String(format!(
//                     "```\ntype {}\n```\n\n---\n\n{}",
//                     node.name,
//                     node.description
//                         .as_ref()
//                         .map(|description| description.to_string())
//                         .unwrap_or_default()
//                 ))),
//                 range: None,
//             });
//         } else {
//             for field in node
//                 .fields_definition
//                 .iter()
//                 .flat_map(|fields| fields.definitions.iter())
//             {
//                 if field.name.span().contains(self.0) {
//                     accumulator.replace(Definition {
//                         contents: DefinitionContents::Scalar(MarkedString::String(format!(
//                             "```\ntype {}\n{}: ...\n```\n\n---\n\n{}",
//                             node.name,
//                             field.name,
//                             field
//                                 .description
//                                 .as_ref()
//                                 .map(|description| description.to_string())
//                                 .unwrap_or_default()
//                         ))),
//                         range: None,
//                     });
//                 }
//             }
//         }
//     }
// }
