extern crate winit;

fn main() {
    let mut events_loop = winit::EventsLoop::new();
    let mut lpressed = false;
    let _window = winit::Window::new(&events_loop).unwrap();

    println!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<inkml:ink xmlns="http://www.w3.org/TR/InkML">
  <inkml:definitions>
    <inkml:context xml:id="ctx0">
      <inkml:inkSource xml:id="inkSrc0">
        <inkml:traceFormat>
          <inkml:channel name="X" type="decimal">
          <inkml:channel name="Y" type="decimal">
        </inkml:traceFormat>
    </inkml:context>
    <inkml:brush xml:id="br0">
      <inkml:brushProperty name="width" value=".07" units="mm"/>
      <inkml:brushProperty name="color" value="\#00AA24" />
    </inkml:brush>
  </inkml:definitions>
  <inkml:traceGroup>"#);

    events_loop.run_forever(|event| {
        // println!("{:?}", event);
        use winit::Event::WindowEvent;
        use winit::ElementState;
        use winit::WindowEvent::*;
        use winit::ControlFlow;
        
        match event {
            WindowEvent {
                event: CloseRequested,
                ..
            } => ControlFlow::Break,
            
            /*
            WindowEvent {
                event: winit::WindowEvent::Touch(touch),
                ..
            } => {
                println("       {} {},", touch.location.x, touch.location.y);
                ControlFlow::Continue
            }
            */
            WindowEvent {
                event: CursorMoved { position, .. },
                ..
            } => {
                if lpressed {
                    println!("      {} {},", position.x, position.y);
                }
                ControlFlow::Continue    
            },

            WindowEvent {
                event: MouseInput { state, button: winit::MouseButton::Left, .. },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        lpressed = true;
                        println!("    <inkml:trace contextRef=\"#ctx0\" brushRef=\"#br0\">");
                    },
                    ElementState::Released => { 
                        lpressed = false;
                        println!("    </inkml:trace>");
                    },
                    
                }
                ControlFlow::Continue
            },
            _ => ControlFlow::Continue
        }
    });
             
    println!("  </inkml:traceGroup>
</inkml:ink>");
}
