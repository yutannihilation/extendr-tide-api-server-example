use extendr_api::graphics::color::Color;
use extendr_api::graphics::{Context, Device, Unit};
use extendr_api::prelude::*;
use extendr_engine::start_r;

use async_std::channel::unbounded;
use async_std::sync::{Arc, Mutex};
use tide::prelude::*;
use tide::Request;

// `Device` is not `Send`, so I needed to use this unsafe marker.
// Is this really needed? Am I in the right direction...?
struct UnsafeDevice(Device);

unsafe impl core::marker::Send for UnsafeDevice {}

impl UnsafeDevice {
    fn new() -> Self {
        Self(Device::current().unwrap())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DeviceOp {
    PlotCircle { x: f64, y: f64, radius: f64 },
    WriteResult,
}

#[derive(Clone)]
struct State {
    sender: Arc<Mutex<async_std::channel::Sender<DeviceOp>>>,
    svg_file: String,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    start_r();

    // Use svg() device as this can be displayed on the web browsers
    let dir = std::env::temp_dir();

    // Path for result SVG
    let path = dir.join("test.svg");
    let path_str = path.to_string_lossy().to_string();

    // Path for temporary SVG file
    let path_tmp = dir.join("tmp.svg");
    let path_tmp_str = path_tmp.to_string_lossy().to_string();

    // For debugging purpose
    println!("{}", &path_str);

    // channel for sending requests to the dedicated thread for R operation.
    let (sender, receiver) = unbounded();

    let state = State {
        sender: Arc::new(Mutex::new(sender)),
        svg_file: path_str.clone(),
    };

    // Create a dedicated thread for calling R function to ensure there's no concurrent R call.
    let _handle = async_std::task::spawn(async move {
        loop {
            println!("Creating a new device...");
            R!("svg({{path_tmp_str.clone()}})").unwrap();
            let dev = UnsafeDevice::new();

            while let Ok(op) = receiver.recv().await {
                match op {
                    // Plot a circle
                    DeviceOp::PlotCircle { x, y, radius } => {
                        println!("Plotting a circle...");
                        let mut gc = Context::from_device(&dev.0, Unit::Inches);

                        // Draw a circle
                        gc.fill(Color::rgb(0x20, 0x20, 0xc0));
                        dev.0.circle((x, y), radius, &gc);
                    }

                    // Write results to a file and break the while loop
                    DeviceOp::WriteResult => {
                        println!("Writing results to file");

                        dev.0.mode_off().unwrap();
                        R!("dev.off()").unwrap();

                        // The file needs to be moved to allow the new device to open the same name of the file.
                        // Use an R call instead of Rust in the hope that the IO operation is finished before moving the file (needs confirmation...)
                        R!("file.rename({{path_tmp_str.clone()}}, {{path_str.clone()}})").unwrap();

                        // Finish the current loop and create a new device
                        break;
                    }
                }
            }
        }
    });

    let mut app = tide::with_state(state);

    app.at("/plot/point").post(plot_point);
    app.at("/plot/result").get(plot_result);

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}

async fn plot_point(mut req: Request<State>) -> tide::Result {
    if let op @ DeviceOp::PlotCircle { .. } = req.body_json().await? {
        println!("Request: {:?}", &op);

        let sender = &mut req.state().sender.lock().await;

        sender.send(op).await?;

        Ok("OK".into())
    } else {
        println!("Invalid request");

        let mut res = tide::Response::new(tide::StatusCode::BadRequest);
        res.set_body("Invalid JSON");
        Ok(res)
    }
}

async fn plot_result(req: Request<State>) -> tide::Result {
    let sender = &mut req.state().sender.lock().await;
    let svg_file = req.state().svg_file.clone();

    sender.send(DeviceOp::WriteResult).await?;

    // TODO: How many seconds is enough for waiting for R's I/O operations?
    //       Ideally, this probably needs to be notified from within the dedicated thread.
    //       But I'm lazy this time.
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Read the result SVG and return it
    let mut res: tide::Response = std::fs::read_to_string(svg_file)?.into();
    res.set_content_type("image/svg+xml");
    res.append_header("Vary", "Accept-Encoding");

    Ok(res)
}
