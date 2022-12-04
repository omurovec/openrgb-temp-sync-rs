extern crate systemstat;

use tokio::{net::TcpStream, time::sleep};

use std::time::Duration;
use systemstat::{System, Platform};
use openrgb::{OpenRGB, data::Color};

pub static UPPER_TEMP: f32 = 85.0;
pub static LOWER_TEMP: f32 = 32.0;

pub static BASE_G_VALUE: u8 = 10;
pub static MAX_C_VALUE: u8 = 20;

#[tokio::main]
async fn main() {
    let sys = System::new();
    let client = match OpenRGB::connect().await {
        Ok(res) => res,
        Err(error) => panic!("Couldn't connect to OpenRgb server: {:?}", error),
    };
    let mut temp = 0.0;

    client.set_name("Rust Temp Sync").await.unwrap();

    println!("Connected using protocol version {}", client.get_protocol_version());

    loop {
        match sys.cpu_temp() {
            Ok(cpu_temp) => {
                println!("\nCPU temp: {}", cpu_temp);
                if cpu_temp != temp {
                    update_color(&temp, &client).await;
                    temp = cpu_temp;
                }
            },
            Err(x) => println!("\nCPU temp: {}", x)
        }
        sleep(Duration::from_millis(500)).await;
    }
}

async fn update_color(temp: &f32, client: &OpenRGB<TcpStream>) {

    let mut r: u8 = 0;
    let temp_scale: f32 = (temp - LOWER_TEMP) / (UPPER_TEMP - LOWER_TEMP);

    if *temp > LOWER_TEMP {
        r = (temp_scale * MAX_C_VALUE as f32) as u8;
    }

    let g: u8 = BASE_G_VALUE;
    let b: u8 = 0;

    let num_controllers = match client.get_controller_count().await {
        Ok(res) => {
            print!("Found {} controllers", res);
            res
        },
        Err(error) => panic!("Couldn't read number of controllers from OpenRgb server: {:?}", error),
    };

    for controller_id in 0..num_controllers {
        match client.update_leds(controller_id, Vec::from([Color{ r, g, b}])).await {
            Ok(()) => (),
            Err(error) => panic!("Couldn't set controller {}: {:?}", controller_id, error)
        };
        print!("Controller {} set to {}", controller_id, Color{r, g, b})
    }
}
