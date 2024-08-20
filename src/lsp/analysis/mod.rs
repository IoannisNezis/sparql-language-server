mod state;

pub use state::*;

use super::textdocument::Position;

pub fn get_kind_at_position(
    analyis_state: &AnalysisState,
    uri: &String,
    position: &Position,
) -> Option<&'static str> {
    match analyis_state.get_tree(uri) {
        Some(tree) => {
            let point = position.to_point();
            Some(
                tree.root_node()
                    .descendant_for_point_range(point, point)?
                    .kind(),
            )
        }
        None => None,
    }
}
