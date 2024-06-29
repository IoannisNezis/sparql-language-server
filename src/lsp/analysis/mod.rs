mod state;

pub use state::*;

use super::textdocument::Position;

pub fn get_token(analyis_state: &AnalysisState, uri: &String, position: &Position) -> String {
    match analyis_state.get_tree(uri) {
        Some(tree) => {
            let point = position.to_point();
            let node = tree
                .root_node()
                .named_descendant_for_point_range(point, point)
                .unwrap();
            node.to_string()
        }
        None => "Tree for document does not exist".to_string(),
    }
}
