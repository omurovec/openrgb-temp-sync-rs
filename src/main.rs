use tokio::{net::TcpStream, time::sleep};

use std::time::Duration;
use openrgb::{OpenRGB, data::Color};

pub static UPPER_TEMP: f64 = 85.0;
pub static LOWER_TEMP: f64 = 32.0;

pub static BASE_C_VALUE: u8 = 10;
pub static MAX_C_VALUE: u8 = 20;

#[tokio::main]
async fn main() {   
    let sensors = match lm_sensors::Initializer::default().initialize() {
        Ok(res) => res,
        Err(error) => panic!("Couldn't start lm_sensors: {}", error),
    };
    let client = match OpenRGB::connect().await {
        Ok(res) => res,
        Err(error) => panic!("Couldn't connect to OpenRgb server: {:?}", error),
    };
    let mut temp: f64 = 0.0;

    client.set_name("Rust Temp Sync").await.unwrap();

    println!("Connected using protocol version {}", client.get_protocol_version());

    loop {
        let mut temp_max = 0;
        for chip in sensors.chip_iter(None) {
            for feature in chip.feature_iter() {
                if matches!(feature.kind(), lm_sensors::feature::Kind.Tempurature) {
                    for sub_feature in feature.sub_feature_iter() {
                        if let Ok(value) = sub_feature.value() {
                            match temp_max {
                                lm_sensors::Value.TemperatureInput(temp) => {
                                    if value > temp_max {
                                        temp_max = max;
                                    }
                                },
                            }
                            temp_max = value;
                        }
                    }
                }
            }
        }
        update_color(temp, &client);
        sleep(Duration::from_millis(500)).await;
    }
}

async fn update_color(temp: &f64, client: &OpenRGB<TcpStream>) {

    let mut r: u8 = BASE_C_VALUE;
    let temp_scale: f64 = (temp - LOWER_TEMP) / (UPPER_TEMP - LOWER_TEMP);

    if *temp > LOWER_TEMP {
        r = (temp_scale * (MAX_C_VALUE - BASE_C_VALUE) as f64) as u8 + BASE_C_VALUE;
    }

    let g: u8 = BASE_C_VALUE;
    let b: u8 = 0;

    let num_controllers = match client.get_controller_count().await {
        Ok(res) => {
            println!("Found {} controllers", res);
            res
        },
        Err(error) => panic!("Couldn't read number of controllers from OpenRgb server: {:?}", error),
    };

    for controller_id in 0..num_controllers {
        match client.update_leds(controller_id, Vec::from([Color{ r, g, b}])).await {
            Ok(()) => (),
            Err(error) => panic!("Couldn't set controller {}: {:?}", controller_id, error)
        };
        println!("Controller {} set to {}", controller_id, Color{r, g, b})
    }
}
