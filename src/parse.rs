use std::error::Error;
use std::io::Read;

use xml::name::OwnedName;
use xml::reader::{EventReader, XmlEvent};

use crate::inkml::{Ink, Trace, TraceGroup, Traces};
use crate::render::Point;

#[derive(Debug)]
pub enum Node {
    Ink(Ink),
    Trace(Trace),
    TraceGroup(TraceGroup),
}

fn parse_vertices(data: String) -> Vec<Point> {
    data.split(',').filter_map(|pos| {
        let xy = pos.split(' ').filter_map(|s| s.parse::<f64>().ok()).collect::<Vec<_>>();
        if xy.len() > 0 {
            Some(Point(xy[0].clone(), xy[1].clone() ))
        } else {
            None
        }
    }).collect::<Vec<_>>()
}

pub fn parse_inkml<R: Read>(inkml: R) -> Result<Ink, Box<dyn Error>> {
    let parser = EventReader::new(inkml);

    let mut name_stack: Vec<String> = vec![]; // Names of elements
    let mut parse_stack: Vec<Node> = vec![]; // built nodes not part of the tree yet
    let mut root: Ink = Default::default();

    for e in parser {
        match e? {
            XmlEvent::StartElement {
                name: OwnedName { ref local_name, .. },
                ..
            } => {
                name_stack.push(local_name.clone());

                match &local_name[..] {
                    "ink" => {
                        parse_stack.push(Node::Ink(Default::default()))
                    }
                    "trace" => { 
                        parse_stack.push(Node::Trace(Default::default()))
                    }
                    "traceGroup" => { 
                        parse_stack.push(Node::TraceGroup(Default::default()))
                    }
                    _ => {}
                }
            }

            XmlEvent::EndElement {
                name: OwnedName { ref local_name, .. },
                ..
            } => {
                // Check to see if we can attach this to a parent node
                if ["ink", "trace", "traceGroup"]
                    .contains(&&local_name[..]) 
                    {
                        let node = parse_stack.pop().unwrap();
                        let top = parse_stack.last_mut();
                        
                        name_stack.pop(); // Remove our name from the list of nodes

                        match node {
                            Node::Ink(ink) => {
                                if let None = top {
                                    root = ink;
                                }
                            }
                            
                            Node::Trace(trace) => {
                                match top {
                                    Some(&mut Node::Ink(Ink { ref mut traces })) 
                                    | Some(&mut Node::TraceGroup(TraceGroup { ref mut traces, .. })) => {
                                        traces.push(Traces::Trace(trace));
                                    }
                                    _ => { }
                                }
                            }
                            
                            Node::TraceGroup(trace_group) => {
                                match top {
                                    Some(&mut Node::Ink(Ink { ref mut traces })) 
                                    | Some(&mut Node::TraceGroup(TraceGroup { ref mut traces, .. })) => {
                                        traces.push(Traces::TraceGroup(trace_group));
                                    }
                                    _ => { } 
                                }
                            }
                            
                            _ => {}
                        }
                        
                    }
            }

            XmlEvent::Characters(contents) => {
                match (name_stack.last().map(|s| &s[..]), parse_stack.last_mut()) {
                    (Some("trace"),
                     Some(&mut Node::Trace(Trace { ref mut vertices }))) => {
                        vertices.append(&mut parse_vertices(contents));
                    }

                    _ => { }
                }
        
            }
            _ => {}
        }
    }

    Ok(root)
}
