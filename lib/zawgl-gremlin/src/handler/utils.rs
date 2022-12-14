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

use zawgl_core::{graph::{EdgeIndex, traits::{GraphContainerTrait, GraphTrait}}, model::{Node, Property, PropertyGraph, PropertyValue, Relationship, Status, predicates::{NamedPropertyPredicate, PropertyPredicate}}};
use zawgl_tx_handler::{ResultGraph, tx_handler::Scenario};

use super::{GremlinError, steps::gremlin_state::StateContext};

use super::super::gremlin::*;


pub fn init_pattern(context: &mut StateContext, n: Node) {
    let mut pattern = PropertyGraph::new();
    let nid = pattern.add_node(n);
    context.patterns.push(pattern);
    context.node_index = Some(nid);
}

pub fn prop_value_from_gremlin_value(gval: &GValue) -> PropertyValue {
    match gval {
        GValue::String(sval) => {
            PropertyValue::PString(sval.clone())
        }
        GValue::Bool(bval) => {
            PropertyValue::PBool(*bval)
        }
        GValue::Integer(ival) => {
            match ival {
                GInteger::I32(ivalue) => {
                    PropertyValue::PInteger(ivalue.0 as i64)
                },
                GInteger::I64(ivalue) => {
                    PropertyValue::PInteger(ivalue.0)
                }
            }
        }
        GValue::Double(dval) => {
            PropertyValue::PFloat(dval.0)
        }
    }
}


pub fn gremlin_value_from_property_value(p: &Property) -> Option<(GInt64, GValue)> {
    Some((GInt64(p.get_id()? as i64),
        match p.get_value() {
            PropertyValue::PString(v) => GValue::String(v.clone()),
            PropertyValue::PInteger(v) => GValue::Integer(GInteger::I64(GInt64(*v))),
            PropertyValue::PFloat(v) => GValue::Double(GDouble(*v)),
            PropertyValue::PBool(v) => GValue::Bool(*v),
        }))
}


pub fn convert_gremlin_predicate_to_pattern_predicate(name: &str, predicate: &GPredicate) -> NamedPropertyPredicate {
    match predicate {
        GPredicate::Value(v) => {
            NamedPropertyPredicate::new(name, PropertyPredicate::EqualTo(prop_value_from_gremlin_value(v)))
        },
        GPredicate::Within(list) => {
            let props = list.values.iter().map(|v| prop_value_from_gremlin_value(v)).collect();
            NamedPropertyPredicate::new(name, PropertyPredicate::Contain(props))
        },
    }
}



fn build_vertex_from_node(n: &Node) -> Option<GVertex> {
    let label = n.get_labels_ref().join(":");
    let id = GValue::Integer(GInteger::I64(GInt64(n.get_id()? as i64)));
    Some(GVertex{id: id, label: label, properties: build_node_properties(n)?})
}

fn build_node_properties(n: &Node) -> Option<GProperties> {
    if n.get_status() != &Status::Create {
        let mut list = Vec::new();
        for p in n.get_properties_ref() {
            list.push(build_property(p)?)
        }
        Some(GProperties { properties: list})
    } else {
        Some(GProperties { properties: vec![]})
    }
}


fn build_edge_properties(r: &Relationship) -> Option<GProperties> {
    if r.get_status() != &Status::Create {
        let mut list = Vec::new();
        for p in r.get_properties_ref() {
            list.push(build_property(p)?)
        }
        Some(GProperties { properties: list})
    } else {
        Some(GProperties { properties: vec![]})
    }
}

fn build_property(p: &Property) -> Option<GProperty> {
    Some(GProperty { name: String::from(p.get_name()), values: vec![gremlin_value_from_property_value(p)?]})
}

pub fn convert_graph_to_gremlin_response(graphs: &Vec<PropertyGraph>, request_id: &str) -> Result<GremlinResponse, GremlinError> {
    let mut res = GResult::new();
    for graph in graphs {
        for n in graph.get_nodes() {
            let vertex = build_vertex_from_node(n).ok_or_else(|| GremlinError::ResponseError)?;
            let traverser = GTraverser{bulk: GInt64(1), value: GItem::Vertex(vertex)};
            res.data.values.push(traverser);
        }

        let mut r_index = 0;
        for r in graph.get_relationships() {
            let edge_index = EdgeIndex::new(r_index);
            let s_index = graph.get_source_index(&edge_index);
            let t_index = graph.get_target_index(&edge_index);
            let label = r.get_labels_ref().join(":");
            let id = GInt64(r.get_id().ok_or_else(|| GremlinError::ResponseError)? as i64);
            let source = graph.get_node_ref(&s_index);
            let target = graph.get_node_ref(&t_index);
            let edge = GEdge{id: id, label: label, 
                out_v_abel: target.get_labels_ref().join(":"),
                in_v_label: source.get_labels_ref().join(":"),
                in_v: GInt64(source.get_id().ok_or_else(|| GremlinError::ResponseError)? as i64),
                out_v: GInt64(target.get_id().ok_or_else(|| GremlinError::ResponseError)? as i64),
                properties: build_edge_properties(r).ok_or_else(|| GremlinError::ResponseError)?,
            };
            let traverser = GTraverser{bulk: GInt64(1), value: GItem::Edge(edge)};
            res.data.values.push(traverser);
            r_index += 1;
        }
    }
    
    let attrs = GMap::new();
    Ok(GremlinResponse{request_id: String::from(request_id), status: GStatus{message: String::from(""), code: 200, attributes: attrs}, result: res})
}