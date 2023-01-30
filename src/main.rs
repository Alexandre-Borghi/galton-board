use core::f64;
use log::{debug, info};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window};

const WIDTH: f64 = 1280.;
const HEIGHT: f64 = 720.;
const PIN_RADIUS: f64 = 5.;
const PIN_INTERVAL: f64 = 35.;
const ROW_COUNT: usize = 15;

struct App {
    ctx: CanvasRenderingContext2d,
    last_frame_ms: f64,
    choices: Vec<Vec<PinChoices>>,
}

#[derive(Debug, Default, Clone, Copy)]
struct PinChoices {
    times_left: u64,
    times_right: u64,
}

fn main() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).unwrap();
    let document = window().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .query_selector("canvas")?
        .unwrap()
        .dyn_into()
        .unwrap();
    canvas.set_width(WIDTH as u32);
    canvas.set_height(HEIGHT as u32);
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into()
        .unwrap();
    let choices = (0..ROW_COUNT - 1)
        .map(|i| (0..=i).map(|_| PinChoices::default()).collect())
        .collect();
    let mut app = App {
        ctx,
        last_frame_ms: window().performance().unwrap().now(),
        choices,
    };
    log::info!("{:?}", app.choices);
    let cb_loop = Rc::new(RefCell::new(None));
    let cb_init = cb_loop.clone();
    *cb_init.borrow_mut() = Some(Closure::new(Box::new(move |t: f64| {
        draw(t, &mut app)?;
        request_animation_frame(cb_loop.borrow().as_ref().unwrap());
        Ok(())
    })));
    request_animation_frame(cb_init.borrow().as_ref().unwrap());
    Ok(())
}

fn request_animation_frame(f: &Closure<dyn FnMut(f64) -> Result<(), JsValue>>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn window() -> Window {
    web_sys::window().unwrap()
}

fn draw(t: f64, app: &mut App) -> Result<(), JsValue> {
    let dt = (t - app.last_frame_ms) / 1000.;
    app.last_frame_ms = t;
    app.clear_background();
    app.draw_pins()?;

    let mut current_pin = 0;
    for i in 0..ROW_COUNT - 1 {
        let goes_left: bool = rand::random();
        if goes_left {
            app.choices[i][current_pin].times_left += 1;
        } else {
            app.choices[i][current_pin].times_right += 1;
            current_pin += 1;
        }
    }

    Ok(())
}

impl App {
    pub fn clear_background(&self) {
        self.ctx.set_fill_style(&"rgb(51, 51, 51)".into());
        self.ctx.fill_rect(0., 0., WIDTH, HEIGHT);
    }

    pub fn draw_pins(&self) -> Result<(), JsValue> {
        for i in 0..15 {
            self.draw_row(i)?;
        }
        Ok(())
    }

    pub fn draw_row(&self, n: u32) -> Result<(), JsValue> {
        let y = PIN_INTERVAL * n as f64 + 100.;
        let x_start = WIDTH / 2. - (n as f64 / 2.) * PIN_INTERVAL;
        for i in 0..=n {
            self.draw_pin(x_start + i as f64 * PIN_INTERVAL, y)?;
        }
        Ok(())
    }

    pub fn draw_pin(&self, x: f64, y: f64) -> Result<(), JsValue> {
        self.ctx.set_fill_style(&"white".into());
        self.ctx.begin_path();
        self.ctx
            .ellipse(x, y, PIN_RADIUS, PIN_RADIUS, 0., 0., f64::consts::TAU)?;
        self.ctx.fill();
        Ok(())
    }
}
