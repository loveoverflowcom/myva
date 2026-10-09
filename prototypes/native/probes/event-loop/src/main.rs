//! Tests a specific rejected lifecycle: construct/drop/reconstruct a winit loop.
//! This desktop source probe is not an Android/iOS runtime result.
use winit::{error::EventLoopError, event_loop::EventLoop};

fn main() {
    let first = EventLoop::new().expect("First event loop needs a working display (or xvfb-run)");
    println!("first_event_loop=created");
    drop(first);
    match EventLoop::new() {
        Err(EventLoopError::RecreationAttempt) => {
            println!("second_event_loop=RecreationAttempt (expected ownership constraint)");
        }
        Err(other) => panic!("Unexpected error; not ownership evidence: {other:?}"),
        Ok(_) => panic!("Event loop recreation unexpectedly succeeded; revisit ADR"),
    }
}
