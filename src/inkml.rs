use std::collections::VecDeque;

use crate::parse::Point;

#[derive(Debug, Default)]
pub struct Ink {
    pub traces: Vec<Traces>,
}

#[derive(Debug, Default)]
pub struct Trace {
    pub vertices: Vec<Point>
}

#[derive(Debug, Default)]
pub struct TraceGroup {
    pub traces: Vec<Traces>,
}

#[derive(Debug)]
pub enum Traces {
    Trace(Trace),
    TraceGroup(TraceGroup)
}

#[derive(Debug)]
pub enum Node<'a> {
    Ink(&'a Ink),
    Traces(&'a Traces),
}

pub struct NodeIter<'a> {
    queue: VecDeque<Node<'a>>,
}

impl<'a> Iterator for NodeIter<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Node<'a>> {
        let next = self.queue.pop_front();

        match next {
            Some(Node::Ink(ref ink)) => {
                self.queue.extend(ink.traces.iter().map(|tg| Node::Traces(tg)));
            }

            Some(Node::Traces(ref trace_node)) => {
                match trace_node {
                    Traces::TraceGroup(ref trace_group) => {
                        self.queue.extend(trace_group.traces.iter() .map(|tg| Node::Traces(tg)));
                    }
                    _ => { }
                }
            }
            
            _ => { }
        }

        next
    }
}

impl<'a> Ink {
    pub fn iter(&'a self) -> NodeIter<'a> {
        (&self).into_iter()
    }

    // Add a new trace to the document
    pub fn draw(&mut self, trace: &Vec<Point>) {
       self.traces.push(Traces::Trace(Trace { vertices: trace.clone() }));
    }
}

impl<'a> IntoIterator for &'a Ink {
    type Item = Node<'a>;
    type IntoIter = NodeIter<'a>;

    fn into_iter(self) -> NodeIter<'a> {
        let mut queue = VecDeque::new();
        queue.push_back(Node::Ink(&self));
            
        NodeIter {
            queue: queue,
        }
    }
}
