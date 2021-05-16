use super::{State, StateContext, match_vertex_state::MatchVertexState};
use one_graph_gremlin::gremlin::*;
use super::gremlin_state::*;

pub struct FromState {
    alias: String,
}

impl FromState {
    pub fn new(alias: &str) -> Self {
        FromState{alias: alias.clone()}
    }
}
impl State for FromState {
    fn handle_step(&self, step: &GStep, context: &mut StateContext) -> Result<Box<dyn State>, StateError> {
        
        match step {
            GStep::V(vid) => {
                Ok(Box::new(MatchVertexState::new(vid)))
            }
            _ => {
                Err(StateError::Invalid)
            }
        }
    }
}