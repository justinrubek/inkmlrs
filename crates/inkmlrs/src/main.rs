extern crate ggez;
extern crate structopt;
extern crate xml;

mod error;
mod inkml;
mod parse;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use structopt::StructOpt;

use ggez::event::{self, KeyCode, KeyMods};
use ggez::graphics::{self, Mesh};
use ggez::*;

use self::inkml::Ink;

macro_rules! draw_trace {
    ($ctx:expr, $vertices:expr) => {
        for pts in $vertices.windows(2) {
            match Mesh::new_line(
                $ctx,
                &[[pts[0][0], pts[0][1]], [pts[1][0], pts[1][1]]],
                5.0,
                graphics::Color::WHITE,
            ) {
                Ok(m) => {
                    graphics::draw($ctx, &m, graphics::DrawParam::default()).unwrap();
                }
                Err(e) => panic!("{:?}", e),
            }
        }
    };
}

#[derive(Debug, StructOpt)]
#[structopt(name = "inkml-rs", about = "Draw lines using InkML")]
struct Opt {
    #[structopt(long, short, parse(from_os_str))]
    /// The file to read from initially
    input: Option<PathBuf>,

    #[structopt(long, short, parse(from_os_str))]
    /// The file to save to. Defaults to stdout if not present
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    let document: Ink = {
        if let Some(path) = opt.input {
            let file = File::open(path)?;
            let file = BufReader::new(file);
            parse::parse_inkml(file)?
        } else {
            Default::default()
        }
    };

    // let ctx = &mut Context::load_from_conf("inkmlrender", "justinrubek", c)?;
    let (mut ctx, event_loop) = ContextBuilder::new("inkmlrender", "justinrubek")
        .with_conf_file(true)
        .build()?;

    let mut state = State::new(&mut ctx, document, opt.output)?;
    state.draw_document(&mut ctx);

    event::run(ctx, event_loop, state);
}

struct State {
    document: Option<Ink>,
    canvas: graphics::Canvas,
    pos_x: f32,
    pos_y: f32,
    mouse_down: bool,
    current_trace: Vec<[f32; 2]>, // The trace currently being drawn by the user
    current_trace_all: Vec<[f32; 2]>, // A buffer used to keep the entirety of the current trace as an optimization
    output_filename: Option<PathBuf>,
}

impl State {
    fn new(ctx: &mut Context, document: Ink, out_file: Option<PathBuf>) -> GameResult<State> {
        let canvas = graphics::Canvas::with_window_size(ctx)?;

        Ok(State {
            document: Some(document),
            canvas,
            pos_x: 100.0,
            pos_y: 100.0,
            mouse_down: false,
            current_trace: Vec::new(),
            current_trace_all: Vec::new(),
            output_filename: out_file,
        })
    }

    fn draw_document(&mut self, ctx: &mut Context) {
        if let Some(ink) = &self.document {
            graphics::set_canvas(ctx, Some(&self.canvas));

            ink.iter().for_each(|n| {
                if let inkml::Node::Traces(inkml::Traces::Trace(inkml::Trace { ref vertices })) = n
                {
                    draw_trace!(ctx, vertices)
                }
            });

            graphics::set_canvas(ctx, None);
        }
    }
}

impl event::EventHandler<ggez::GameError> for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::Color::BLACK);

        // Draw the canvas containing the already drawn inkml to the screen
        graphics::draw(ctx, &self.canvas, graphics::DrawParam::default())?;

        // Draw the 'current' trace (currently being drawn by user)
        draw_trace!(ctx, self.current_trace);

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.mouse_down = true;
    }

    fn mouse_button_up_event(
        &mut self,
        ctx: &mut Context,
        _button: event::MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.mouse_down = false;

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

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32, _xrel: f32, _yrel: f32) {
        if self.mouse_down {
            self.pos_x = x;
            self.pos_y = y;
            self.current_trace.push([self.pos_x, self.pos_y]);
        }
        if self.current_trace.len() > 300 {
            graphics::set_canvas(ctx, Some(&self.canvas));
            draw_trace!(ctx, self.current_trace);
            graphics::set_canvas(ctx, None);
            self.current_trace_all.append(&mut self.current_trace);
        }
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        repeat: bool,
    ) {
        if keycode == KeyCode::W && !repeat {
            if let Some(ink) = &mut self.document {
                if let Some(filename) = &self.output_filename {
                    println!("Writing to file {filename:?}");
                    ink.write_to(&mut File::create(filename).unwrap()).unwrap();
                } else {
                    ink.write_to(&mut std::io::stdout()).unwrap();
                }
            }
        }
    }
}
