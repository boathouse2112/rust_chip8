use std::f64;
use wasm_bindgen::prelude::*;
use web_sys::{console, CanvasRenderingContext2d, Element, HtmlCanvasElement};

// println!-like macro for console.log()
macro_rules! console_log {
    ($($t:tt)*) => (console::log_1(&format_args!($($t)*).to_string().into()))
}
/// Adds .expect_log extension method that unwraps an Option/Result, or logs the given message and panics if it can't.
trait ExpectLog: Sized {
    type T;
    fn expect_log(self, msg: &str) -> Self::T;
}

impl<T> ExpectLog for Option<T> {
    type T = T;

    fn expect_log(self, msg: &str) -> Self::T {
        match self {
            Some(val) => return val,
            None => {
                console_log!("{}", msg);
                panic!();
            }
        }
    }
}

impl<T, E> ExpectLog for Result<T, E> {
    type T = T;

    fn expect_log(self, msg: &str) -> Self::T {
        match self {
            Ok(val) => return val,
            Err(_) => {
                console_log!("{}", msg);
                panic!();
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let canvas = document
        .get_element_by_id("canvas")
        .expect_log("can't find element #canvas");

    let canvas: HtmlCanvasElement = canvas.dyn_into().map_err(|_| ()).unwrap();

    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();

    context.begin_path();

    // Draw the outer circle.
    context
        .arc(75.0, 75.0, 50.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the mouth.
    context.move_to(110.0, 75.0);
    context.arc(75.0, 75.0, 35.0, 0.0, f64::consts::PI).unwrap();

    // Draw the left eye.
    context.move_to(65.0, 65.0);
    context
        .arc(60.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    // Draw the right eye.
    context.move_to(95.0, 65.0);
    context
        .arc(90.0, 65.0, 5.0, 0.0, f64::consts::PI * 2.0)
        .unwrap();

    context.stroke();

    Ok(())
}
