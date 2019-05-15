extern crate ggez;
extern crate xml;

use std::sync::Arc;

mod inkml;
mod parse;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use ggez::*;
use ggez::graphics::{DrawMode, Point2};

use self::inkml::{Ink,Node,Traces};
use self::parse::Point;

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("file1.inkml")?;
    let file = BufReader::new(file);
    let document = parse::parse_inkml(file)?;
    // document.iter().for_each(|n| println!("{:?}", n));
    //let document: Ink = Default::default();

    let c = conf::Conf::new();
    let ctx = &mut Context::load_from_conf("super_simple", "ggez", c)?;
    let state = &mut State::new(ctx)?;

    state.ink(document);

    event::run(ctx, state)?;
    // draw(document)
    Ok(())
}

struct State {
    document: Option<Ink>,
}

impl State {
    fn new(_ctx: &mut Context) -> GameResult<State> {
        Ok(State { document: None })
    }

    fn ink(&mut self, document: Ink) {
        self.document = Some(document);
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx);

        // Collect traces into individual segment groups (each stroke)
        if let Some(ink) = &self.document {
            ink.iter().for_each(|n| {
                match n {
                    inkml::Node::Traces(inkml::Traces::Trace(inkml::Trace { ref vertices })) => {
                        for pts in vertices.windows(2) {
                            graphics::line(ctx, 
                                           &[Point2::new(pts[0][0], pts[0][1]), Point2::new(pts[1][0], pts[1][1])],
                                           5.0);
                        }
                    }
                    _ => {}
                }
            });
        }
        graphics::present(ctx);
        Ok(())
    }
}
