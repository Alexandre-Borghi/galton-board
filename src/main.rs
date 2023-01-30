use core::f64;
use log::{debug, info};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, Window};

const WIDTH: f64 = 1280.;
const HEIGHT: f64 = 720.;
const PIN_RADIUS: f64 = 5.;
const PIN_INTERVAL: f64 = 35.;

struct App {
    ctx: CanvasRenderingContext2d,
    last_frame_ms: f64,
    pin_x: f64,
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
    let mut app = App {
        ctx,
        last_frame_ms: window().performance().unwrap().now(),
        pin_x: 0.,
    };
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
    app.pin_x += dt * 10.;
    app.draw_pin(
        (app.pin_x).cos() * 100. + 200.,
        (app.pin_x).sin() * 100. + 200.,
    )?;
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

    pub fn draw_random_path(&self) -> Result<(), JsValue> {
        Ok(())
    }
}
