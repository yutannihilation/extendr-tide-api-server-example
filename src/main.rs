use extendr_api::graphics::color::Color;
use extendr_api::graphics::{Context, Device, Unit};
use extendr_api::prelude::*;
use extendr_engine::start_r;

use async_std::sync::{Arc, Mutex};
use tide::prelude::*;
use tide::Request;

// `Device` is not `Send`, so I needed to use this unsafe marker.
// Is this really needed? Am I in the right direction...?
struct UnsafeDevice(Device);

unsafe impl core::marker::Send for UnsafeDevice {}

#[derive(Clone)]
struct State {
    dev: Arc<Mutex<UnsafeDevice>>,
    svg_file: String,
}

// Convert JSON inputs into this Rust struct
#[derive(Debug, Deserialize)]
struct Circle {
    x: f64,
    y: f64,
    radius: f64,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    start_r();

    // Use svg() device as this can be displayed on the web browsers
    let dir = std::env::temp_dir();
    let path = dir.join("test.svg");
    let path_str = path.to_string_lossy().to_string();

    // For debugging purpose
    println!("{}", &path_str);

    // Create a new device and move it to a `State`
    R!("svg({{path_str.clone()}})").unwrap();
    let dev = Device::current().unwrap();

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

    let dev = &mut req.state().dev.lock().await.0;
    let mut gc = Context::from_device(&dev, Unit::Inches);

    // Draw a circle
    gc.fill(Color::rgb(0x20, 0x20, 0xc0));
    dev.circle((x, y), radius, &gc);

    Ok("OK".into())
}

async fn plot_result(req: Request<State>) -> tide::Result {
    let dev = &mut req.state().dev.lock().await.0;
    let svg_file = req.state().svg_file.clone();

    // Write out the result. Note that, as the device is closed here,
    // further requests will cause panic.
    dev.mode_off().unwrap();
    R!("dev.off()").unwrap();

    // Read the result SVG and return it
    let mut res: tide::Response = std::fs::read_to_string(svg_file)?.into();
    res.set_content_type("image/svg+xml");
    res.append_header("Vary", "Accept-Encoding");

    Ok(res)
}
