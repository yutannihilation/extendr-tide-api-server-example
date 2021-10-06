use extendr_api::graphics::color::Color;
use extendr_api::graphics::{Context, Device, Unit};
use extendr_api::prelude::*;
use extendr_engine::start_r;

use std::sync::{Arc, Mutex};
use tide::prelude::*;
use tide::Request;

struct UnsafeDevice(Device);

unsafe impl core::marker::Send for UnsafeDevice {}

#[derive(Clone)]
struct State {
    dev: Arc<Mutex<UnsafeDevice>>,
    svg_file: String,
}

#[derive(Debug, Deserialize)]
struct Circle {
    x: f64,
    y: f64,
    radius: f64,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    start_r();

    let dir = std::env::temp_dir();
    let path = dir.join("test.svg");
    let path_str = path.to_string_lossy().to_string();

    println!("{}", &path_str);
    R!("svg({{path_str.clone()}})").unwrap();
    let dev = Device::current().unwrap();
    println!("{:?}", dev);

    let state = State {
        dev: Arc::new(Mutex::new(UnsafeDevice(dev))),
        svg_file: path_str,
    };

    let mut app = tide::with_state(state);

    app.at("/plot/point").post(plot_point);
    app.at("/plot/result").get(plot_result);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn plot_point(mut req: Request<State>) -> tide::Result {
    let Circle { x, y, radius } = req.body_json().await?;

    let dev = &mut req.state().dev.lock().unwrap().0;
    let mut gc = Context::from_device(&dev, Unit::Inches);

    let svg_file = req.state().svg_file.clone();

    // Draw a circle
    gc.fill(Color::rgb(0x20, 0x20, 0xc0));
    dev.circle((x, y), radius, &gc);

    Ok("OK".into())
}

async fn plot_result(mut req: Request<State>) -> tide::Result {
    let dev = &mut req.state().dev.lock().unwrap().0;
    let svg_file = req.state().svg_file.clone();

    dev.mode_off();
    R!("dev.off()").unwrap();

    let mut res: tide::Response = std::fs::read_to_string(svg_file)?.into();
    res.set_content_type("image/svg+xml");
    res.append_header("Vary", "Accept-Encoding");

    Ok(res)
}
