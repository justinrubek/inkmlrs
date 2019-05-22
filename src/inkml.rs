use std::collections::VecDeque;
use std::error::Error;
use std::io::Write;

use xml::{EmitterConfig,EventWriter};
use xml::writer::events::XmlEvent;

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
                        self.queue.extend(trace_group.traces.iter().map(|tg| Node::Traces(tg)));
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

    fn write_trace<W: Write>(w: &mut xml::EventWriter<W>, trace: &Trace) {
        w.write(XmlEvent::start_element("trace").attr("contextRef", "#ctx0").attr("brushRef", "#br0"));
        
        for point in &trace.vertices {
            w.write(XmlEvent::characters(&format!("{} {},", point[0], point[1])));
        }
        
        w.write(XmlEvent::end_element());
    }

    fn write_tracegroup<W: Write>(w: &mut xml::EventWriter<W>, group: &TraceGroup) {
        w.write(XmlEvent::start_element("traceGroup").attr("contextRef", "#ctx0").attr("brushRef", "#br0"));
        for traces in &group.traces {
            Ink::write_traces(w, &traces);
        }
        w.write(XmlEvent::end_element());
    }

    fn write_traces<W: Write>(w: &mut xml::EventWriter<W>, traces: &Traces) {
        match traces {
            Traces::Trace(trace) => {
                Ink::write_trace(w, &trace);
            }
            Traces::TraceGroup(group) => {
                Ink::write_tracegroup(w, &group);
            }
        }
    }

    fn write_definitions<W: Write>(w: &mut xml::EventWriter<W>, ink: &Ink) {
        w.write(XmlEvent::start_element("definitions"));
        w.write(XmlEvent::start_element("context").attr("xml:id", "ctx0"));
        w.write(XmlEvent::start_element("inkSource").attr("xml:id", "inkSrc0")); 
        
        w.write(XmlEvent::start_element("traceFormat"));
        w.write(XmlEvent::start_element("channel").attr("name", "X").attr("type", "integer")); 
        w.write(XmlEvent::end_element()); // channel
        w.write(XmlEvent::start_element("channel").attr("name", "Y").attr("type", "integer")); 
        w.write(XmlEvent::end_element()); // channel
        
        w.write(XmlEvent::end_element()); // traceFormat
        w.write(XmlEvent::end_element()); // inkSource
        w.write(XmlEvent::end_element()); // context

        w.write(XmlEvent::start_element("brush").attr("xml:id", "br0"));
        w.write(XmlEvent::start_element("brushProperty").attr("name", "width").attr("value", "3").attr("units", "cm"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::start_element("brushProperty").attr("name", "height").attr("value", "3").attr("units", "cm"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::start_element("brushProperty").attr("name", "color").attr("value", "\\#FFFFFF"));
        w.write(XmlEvent::end_element());
        w.write(XmlEvent::end_element()); // brush
       
        w.write(XmlEvent::end_element()); // definitions
    }
    
    pub fn write_to<W: Write>(&mut self, w: &mut W) -> Result<(), Box<dyn Error>> {
        // Set up EventWriter
        let mut writer = EmitterConfig::new().perform_indent(true).create_writer(w);
        
        
        writer.write(XmlEvent::StartDocument {
            version: xml::common::XmlVersion::Version10,
            encoding: Some("UTF-8"),
            standalone: Some(true),
        });
        // Iterate over and write events for inner nodes
        self.iter().for_each(|n| {
            match n {
                Node::Ink(ink) => {
                    writer.write(XmlEvent::start_element("ink").ns("inkml", "https://www.w3.org/TR/InkML"));
                    
                    Ink::write_definitions(&mut writer, &ink);
                }

                Node::Traces(traces) => {
                    Ink::write_traces(&mut writer, &traces);
                }

            }
        });
            
        writer.write(XmlEvent::end_element());
        Ok(())
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
