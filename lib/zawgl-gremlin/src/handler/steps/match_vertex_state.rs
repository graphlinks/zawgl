// MIT License
//
// Copyright (c) 2022 Alexandre RICCIARDI
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use super::gremlin_state::{State, StateContext};
use super::alias_vertex_state::AliasVertexState;
use super::has_property_state::HasPropertyState;
use zawgl_core::model::*;
use super::super::super::gremlin::*;
use super::gremlin_state::*;
use super::match_out_edge_state::MatchOutEdgeState;
use super::match_state::MatchState;
use super::add_edge_state::AddEdgeState;
use std::convert::TryFrom;
use super::super::utils::*;

pub struct MatchVertexState {
    vid: Option<u64>,
}

impl MatchVertexState {
    pub fn new(gid: &Option<GValueOrVertex>) -> Self {
        let vid = gid.as_ref().and_then(|value| match value {
            GValueOrVertex::Value(v) => {u64::try_from(v.clone()).ok()}
            GValueOrVertex::Vertex(vertex) => {u64::try_from(vertex.id.clone()).ok()}
        }).or(None);
        MatchVertexState{vid: vid}
    }
}


impl State for MatchVertexState {
    
    fn handle_step(&self, context: &mut StateContext) -> Result<(), GremlinStateError> {
        let mut n = Node::new();
        n.set_id(self.vid);
        n.set_status(Status::Match);

        match &context.previous_step {
            GStep::As(_alias) => {
                let pattern = context.patterns.last_mut().ok_or(GremlinStateError::WrongContext("missing pattern"))?;
                let nid = pattern.add_node(n);
                context.node_index = Some(nid);
            }
            GStep::Empty => {
                init_pattern(context, n);
            }
            GStep::OutE(labels) => {
                if let Some(node_index) = context.node_index {
                    let mut rel = Relationship::new();
                    rel.set_labels(labels.clone());
                    rel.set_status(Status::Match);
                    let pattern = context.patterns.last_mut().ok_or(GremlinStateError::WrongContext("missing pattern"))?;
                    let nid = pattern.add_node(n);
                    pattern.add_relationship(rel, node_index, nid);
                    context.node_index = Some(nid);
                }
            }
            GStep::AddE(label) => {
                if let Some(node_index) = context.node_index {
                    let mut rel = Relationship::new();
                    rel.set_labels(vec![label.clone()]);
                    rel.set_status(Status::Create);
                    let pattern = context.patterns.last_mut().ok_or(GremlinStateError::WrongContext("missing pattern"))?;
                    let nid = pattern.add_node(n);
                    pattern.add_relationship(rel, node_index, nid);
                    context.node_index = Some(nid);
                }
            }
            _ => {

            }
        }
        Ok(())
    }

    fn create_state(&self, step: &GStep) -> Result<Box<dyn State>, GremlinStateError> {
        
        match step {
            GStep::OutE(_labels) => {
                Ok(Box::new(MatchOutEdgeState::new()))
            }
            GStep::As(alias) => {
                Ok(Box::new(AliasVertexState::new(alias)))
            }
            GStep::Match(bytecodes) => {
                Ok(Box::new(MatchState::new(bytecodes)))
            }
            GStep::AddE(_label) => {
                Ok(Box::new(AddEdgeState::new()))
            }
            GStep::Has(name, predicate) => {
                Ok(Box::new(HasPropertyState::new(name, predicate)))
            }
            _ => {
                Err(GremlinStateError::Invalid(step.clone()))
            }
        }
    }
}

