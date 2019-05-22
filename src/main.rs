extern crate ggez;
extern crate xml;

mod inkml;
mod parse;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use ggez::*;
use ggez::graphics::{self, Point2};

use self::inkml::{Ink};

macro_rules! draw_trace {
    ($ctx:expr, $vertices:expr) => {
        for pts in $vertices.windows(2) {
            match graphics::line($ctx, 
                           &[Point2::new(pts[0][0], pts[0][1]), Point2::new(pts[1][0], pts[1][1])],
                           5.0) {
                Ok(_) => { },
                Err(e) => panic!("{:?}", e)
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("file1.inkml")?;
    let file = BufReader::new(file);
    let document = parse::parse_inkml(file)?;

    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("inkmlrender", "justinrubek", c)?;
    let state = &mut State::new(ctx)?;

    state.ink(document);
    state.draw_document(ctx);

    event::run(ctx, state)?;
    // draw(document)
    Ok(())
}

struct State {
    document: Option<Ink>,
    canvas: graphics::Canvas,
    pos_x: f32,
    pos_y: f32,
    mouse_down: bool,
    current_trace: Vec<[f32; 2]>, // The trace currently being drawn by the user
    current_trace_all: Vec<[f32; 2]>, // A buffer used to keep the entirety of the current trace as an optimization
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let canvas = graphics::Canvas::with_window_size(ctx)?;
        
        Ok(State { 
            document: None, 
            canvas,
            pos_x: 100.0, 
            pos_y: 100.0, 
            mouse_down: false, 
            current_trace: Vec::new(), 
            current_trace_all: Vec::new(),
        }) 
    }

    fn ink(&mut self, document: Ink) {
        self.document = Some(document);
    }

    fn draw_document(&mut self, ctx: &mut Context) {
        if let Some(ink) = &self.document {
            graphics::set_canvas(ctx, Some(&self.canvas));
            
            ink.iter().for_each(|n| {
                match n {
                    inkml::Node::Traces(inkml::Traces::Trace(inkml::Trace { ref vertices })) => {
                        draw_trace!(ctx, vertices)
                    }
                    _ => {}
                }
            });
            
            graphics::set_canvas(ctx, None);
        }
        
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // Draw the canvas containing the already drawn inkml to the screen
        graphics::draw(
            ctx,
            &self.canvas,
            Point2::new(0.0,0.0),
            0.0
        )?;

        // Collect traces into individual segment groups (each stroke)
        // Draw the 'current' trace (currently being drawn by user)
        draw_trace!(ctx, self.current_trace);
        
        graphics::present(ctx);
        Ok(())
    }
    
    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: event::MouseButton, x: i32, y: i32) {
        self.mouse_down = true;
        println!("Mouse button pressed: {:?}, x: {}, y: {}", button, x, y);
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: event::MouseButton, x: i32, y: i32) {
        self.mouse_down = false;
        println!("Mouse button released: {:?}, x: {}, y: {}", button, x, y);

        // Add the newly created trace to our canvas
        graphics::set_canvas(ctx, Some(&self.canvas));
        draw_trace!(ctx, self.current_trace); 
        graphics::set_canvas(ctx, None);
        
        // Commit points to document and remove from 'current' trace
        if let Some(ink) = &mut self.document {
            self.current_trace_all.append(&mut self.current_trace);
            ink.draw(&self.current_trace_all);
        }
        
        self.current_trace_all.clear();
    } 
    
    fn mouse_motion_event(&mut self, ctx: &mut Context, _ms: event::MouseState, x: i32, y: i32, xrel: i32, yrel: i32) {
        if self.mouse_down {
            self.pos_x = x as f32;
            self.pos_y = y as f32;
            self.current_trace.push([self.pos_x, self.pos_y]);
        }
        if self.current_trace.len() > 300 {
            graphics::set_canvas(ctx, Some(&self.canvas));
            draw_trace!(ctx, self.current_trace);
            graphics::set_canvas(ctx, None);
            self.current_trace_all.append(&mut self.current_trace);
        }
        println!(
            "Mouse motion, x: {}, y: {}, relative x: {}, relative y: {}",
            x, y, xrel, yrel
        );
    } 
}

