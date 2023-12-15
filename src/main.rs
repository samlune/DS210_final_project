// graph
use petgraph::algo::dijkstra;
use petgraph::graph::{UnGraph, NodeIndex};
use petgraph::unionfind::UnionFind;
use petgraph::visit::EdgeRef;

// other 
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
// Code // 
fn line_reader<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file= File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
fn average_path(graph: &UnGraph<u32, ()>, start_node: NodeIndex) -> f64 {
    let mut total_path_length= 0;
    let mut path_count= 0;
    for end_node in graph.node_indices() {
        if start_node!= end_node {
            if let Some(path)= dijkstra(graph, start_node, Some(end_node), |_| 1).get(&end_node) {
                total_path_length+= path;
                path_count+= 1;
            }
        }
    }
    if path_count > 0 {
        total_path_length as f64 / path_count as f64
    } else {
        0.0
    }
}
fn main() -> io::Result<()> {
    let edges_file= "../../facebook/348.edges";  
    let mut graph: petgraph::prelude::Graph<u32, (), petgraph::prelude::Undirected>= UnGraph::<u32, ()>::new_undirected();
    let mut node_indices= HashMap::new();
    if let Ok(lines)= line_reader(edges_file) {
        for line in lines {
            if let Ok(edge)= line {
                let nodes: Vec<&str>= edge.split_whitespace().collect();
                if nodes.len()!= 2 {
                    continue;
                }
                let node1= nodes[0].parse::<u32>().unwrap();
                let node2= nodes[1].parse::<u32>().unwrap();
                let index1= *node_indices.entry(node1).or_insert_with(|| graph.add_node(node1));
                let index2= *node_indices.entry(node2).or_insert_with(|| graph.add_node(node2));
                graph.add_edge(index1, index2, ());
            }
        }
    }
    let mut degrees= graph.node_indices()
                          .map(|n| (n, graph.edges(n).count()))
                          .collect::<Vec<_>>();
    degrees.sort_by(|a, b| b.1.cmp(&a.1));
    let influential_nodes: Vec<NodeIndex>= degrees.iter().take(5).map(|(n, _)| *n).collect();

    let mut components= UnionFind::new(graph.node_count());
    for edge in graph.edge_references() {
        components.union(edge.source().index(), edge.target().index());
    }
    let mut component_members= HashMap::new();
    for node in graph.node_indices() {
        let component= components.find(node.index());
        component_members.entry(component).or_insert_with(Vec::new).push(node);
    }
    let isolated_subgraphs: Vec<Vec<NodeIndex>>= component_members
        .values()
        .filter(|nodes| nodes.len() < 500) 
        .cloned()
        .collect();
    let mut subgraph_influential_nodes= Vec::new();
    for subgraph in &isolated_subgraphs {
        if let Some(&most_connected)= subgraph.iter().max_by_key(|&&node| graph.edges(node).count()) {
            subgraph_influential_nodes.push(most_connected);
        }
    }
    for &subgraph_node in &subgraph_influential_nodes {
        let subgraph_avg_path_length= average_path(&graph, subgraph_node);
        for &influential_node in &influential_nodes {
            let entire_network_avg_path_length= average_path(&graph, influential_node);
            if let Some(distance)= dijkstra(&graph, influential_node, Some(subgraph_node), |_| 1).get(&subgraph_node) {
                println!("Distance from whole-network node {:?} to isolated-network node {:?}: {}", influential_node, subgraph_node, distance);
                println!("Average path length from whole-network node {:?}: {:.2}", influential_node, entire_network_avg_path_length);
                println!("Average path length within isolate-network node {:?}: {:.2}", subgraph_node, subgraph_avg_path_length);
            }
        }
    }
    Ok(())
}