use core::f64;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, InputEvent, KeyboardEvent,
    Window,
};

const PIN_RADIUS: f64 = 7.;
const PIN_INTERVAL: f64 = 40.;
const PINS_START_Y: f64 = PIN_INTERVAL;
const ROW_COUNT: usize = 16;
const HEIGHT: f64 = 1080.;
const WIDTH: f64 = (ROW_COUNT + 2) as f64 * PIN_INTERVAL;
// const WIDTH: f64 = 1920.;
const HISTOGRAM_MAX_Y: f64 = PINS_START_Y + ROW_COUNT as f64 * PIN_INTERVAL;

struct App {
    ctx: CanvasRenderingContext2d,
    last_frame_ms: f64,
    choices: Vec<Vec<PinChoices>>,
    frame_time: f64,
    total_paths: u64,
    animation_speed: f64,
}

#[derive(Debug, Default, Clone, Copy)]
struct PinChoices {
    times_left: u64,
    times_right: u64,
}

fn main() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Trace).unwrap();
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
    let choices = (0..ROW_COUNT)
        .map(|i| (0..=i).map(|_| PinChoices::default()).collect())
        .collect();
    let app = Arc::new(Mutex::new(App {
        ctx,
        last_frame_ms: window().performance().unwrap().now(),
        choices,
        frame_time: 0.,
        total_paths: 0,
        animation_speed: 15.,
    }));

    let cb_loop = Rc::new(RefCell::new(None));
    let cb_init = cb_loop.clone();
    {
        let app = app.clone();
        *cb_init.borrow_mut() = Some(Closure::new(move |t: f64| {
            draw(t, &app)?;
            request_animation_frame(cb_loop.borrow().as_ref().unwrap());
            Ok(())
        }));
        request_animation_frame(cb_init.borrow().as_ref().unwrap());
    }

    {
        let app = app.clone();
        let cb = Closure::<dyn Fn(_)>::new(move |e: KeyboardEvent| {
            if e.code() != "Space" {
                return;
            }
            log::debug!("space");
            app.lock().unwrap().reset();
        });
        window().add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }

    {
        let animation_speed_slider: HtmlInputElement = document
            .get_element_by_id("animation-speed")
            .unwrap()
            .dyn_into()?;
        let cb = Closure::<dyn Fn(_)>::new(move |e: InputEvent| {
            let new_val = e
                .target()
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap()
                .value();
            app.lock().unwrap().animation_speed = new_val.parse().unwrap();
        });
        animation_speed_slider
            .add_event_listener_with_callback("input", cb.as_ref().unchecked_ref())?;
        cb.forget();
    }

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

fn draw(t: f64, app: &Arc<Mutex<App>>) -> Result<(), JsValue> {
    let mut app = app.lock().unwrap();
    let dt = (t - app.last_frame_ms) / 1000.;
    app.frame_time += dt;
    if app.frame_time < 1. / app.animation_speed {
        log::trace!("Skip frame");
        return Ok(());
    }
    app.frame_time -= 1. / app.animation_speed;
    app.last_frame_ms = t;
    app.clear_background();
    app.draw_pins()?;

    app.total_paths += 1;
    let mut current_pin = 0;
    for i in 0..ROW_COUNT {
        if i < ROW_COUNT - 1 {
            for j in 0..=i {
                app.draw_segment(i, j, j, app.choices[i][j].times_left as f64 / 300.)?;
                app.draw_segment(i, j, j + 1, app.choices[i][j].times_right as f64 / 300.)?;
            }
        }

        let goes_left: bool = rand::random();
        if goes_left {
            app.choices[i][current_pin].times_left += 1;
            if i < ROW_COUNT - 1 {
                app.draw_segment_with_color(i, current_pin, current_pin, "rgb(255, 51, 51)")?;
            }
        } else {
            app.choices[i][current_pin].times_right += 1;
            if i < ROW_COUNT - 1 {
                app.draw_segment_with_color(i, current_pin, current_pin + 1, "rgb(255, 51, 51)")?;
            }
            current_pin += 1;
        }
    }

    // Draw histogram

    let mut p_max = 0;
    let ps = (0..ROW_COUNT)
        .map(|i| {
            let mut p = 0;
            if i >= 1 {
                p += app.choices[ROW_COUNT - 1][i - 1].times_right;
            }
            if i < ROW_COUNT - 1 {
                p += app.choices[ROW_COUNT - 1][i].times_left;
            }
            if p > p_max {
                p_max = p;
            }
            p
        })
        .collect::<Vec<_>>();

    for i in 0..ROW_COUNT {
        let x = WIDTH / 2. - (ROW_COUNT as f64 / 2.) * PIN_INTERVAL + i as f64 * PIN_INTERVAL;
        let y = HEIGHT - PIN_INTERVAL;
        let w = PIN_INTERVAL / 2.;
        let h_max = HISTOGRAM_MAX_Y - y;
        let p = ps[i];
        let h = (p as f64 / p_max as f64) * h_max;

        app.ctx.fill_rect(x - w / 2., y, w, h);
    }

    Ok(())
}

impl App {
    pub fn clear_background(&self) {
        self.ctx.set_fill_style(&"rgb(51, 51, 51)".into());
        self.ctx.fill_rect(0., 0., WIDTH, HEIGHT);
    }

    pub fn draw_pins(&self) -> Result<(), JsValue> {
        for i in 0..ROW_COUNT {
            self.draw_row(i)?;
        }
        Ok(())
    }

    pub fn draw_row(&self, n: usize) -> Result<(), JsValue> {
        let y = PIN_INTERVAL * n as f64 + PINS_START_Y;
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

    pub fn draw_segment(
        &self,
        row: usize,
        pin_a: usize,
        pin_b: usize,
        alpha: f64,
    ) -> Result<(), JsValue> {
        self.draw_segment_with_color(row, pin_a, pin_b, &format!("rgb(255, 255, 255, {alpha})"))?;
        Ok(())
    }

    pub fn draw_segment_with_color(
        &self,
        row: usize,
        pin_a: usize,
        pin_b: usize,
        color: &str,
    ) -> Result<(), JsValue> {
        let pin_a_x = WIDTH / 2. - (row as f64 / 2.) * PIN_INTERVAL + pin_a as f64 * PIN_INTERVAL;
        let pin_a_y = PIN_INTERVAL * row as f64 + PINS_START_Y;
        let pin_b_x =
            WIDTH / 2. - ((row + 1) as f64 / 2.) * PIN_INTERVAL + pin_b as f64 * PIN_INTERVAL;
        let pin_b_y = PIN_INTERVAL * (row + 1) as f64 + PINS_START_Y;
        self.ctx.set_stroke_style(&color.into());
        self.ctx.set_line_width(3.);
        self.ctx.begin_path();
        self.ctx.move_to(pin_a_x, pin_a_y);
        self.ctx.line_to(pin_b_x, pin_b_y);
        self.ctx.stroke();
        Ok(())
    }

    fn reset(&mut self) {
        self.choices = (0..ROW_COUNT)
            .map(|i| (0..=i).map(|_| PinChoices::default()).collect())
            .collect();
        self.total_paths = 0;
    }
}
