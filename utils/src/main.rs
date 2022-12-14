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

use std::collections::HashSet;

use zawgl_core::graph::traits::GrowableGraphTrait;
use zawgl_core::graph_engine::model::{GraphProxy, ProxyNodeId};
use zawgl_core::model::init::InitContext;
use zawgl_core::graph_engine::GraphEngine;
use zawgl_core::graph::traits::*;

fn main() {
    //let main_dir = get_tmp_dir_path("zawgl-db");
    let main_dir = "zawgl-db";
    let conf = InitContext::new(&main_dir).expect("can't create context");
    let mut graph_engine = GraphEngine::new(&conf);
    let mut full_graph = graph_engine.retrieve_graph().unwrap();
    println!("{:?}", full_graph.get_nodes_ids());
    depth_first_search(&mut full_graph);
    println!("full_graph {{");
    for e in full_graph.get_edges_with_relationships() {
        let src = full_graph.get_node_ref(&e.0.source).expect("source").get_labels_ref().join(":");
        let trg = full_graph.get_node_ref(&e.0.target).expect("target").get_labels_ref().join(":");
        println!("{:?}[{:?}]--{:?}[{:?}](in:{:?}, out:{:?})-->{:?}[{:?}]", 
        e.0.source.store_id, src, e.1.get_id().unwrap(), e.1.get_labels_ref().join(":"), e.0.next_inbound_edge, e.0.next_outbound_edge,
        e.0.target.store_id, trg);
    }
    println!("}}");
}

fn depth_first_search(graph: &mut GraphProxy) {
    let mut labeled = HashSet::new();
    for id in graph.get_nodes_ids() {
        if !labeled.contains(&id) {
            iterate_adjacent_nodes(&mut labeled, graph, &id);
        }
    }
}

fn iterate_adjacent_nodes(labeled: &mut HashSet<ProxyNodeId>, graph: &mut GraphProxy, id: &ProxyNodeId) {
    labeled.insert(*id);
    println!("{:?}", graph.get_node_ref(&id));
    for e_in in graph.in_edges(&id) {
        println!("{:?}", graph.get_relationship_ref(&e_in));
        let in_v = graph.get_source_index(&e_in);
        if !labeled.contains(&in_v) {
            iterate_adjacent_nodes(labeled, graph, &in_v);
        }
    }
    for e_out in graph.out_edges(&id) {
        println!("{:?}", graph.get_relationship_ref(&e_out));
        let out_v = graph.get_target_index(&e_out);
        if !labeled.contains(&out_v) {
            iterate_adjacent_nodes(labeled, graph, &out_v);
        }
    }
}