use tokio::{net::TcpStream, time::sleep};

use std::time::Duration;
use std::cmp::max;
use openrgb::{OpenRGB, data::Color};
use lm_sensors::{Value::TemperatureInput, feature::Kind::Temperature, chip::SharedChip};

pub static UPPER_TEMP: f64 = 80.0;
pub static LOWER_TEMP: f64 = 32.0;

pub static BASE_C_VALUE: u8 = 20;
pub static MAX_C_VALUE: u8 = 30;

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

    client.set_name("Rust Temp Sync").await.unwrap();

    println!("Connected using protocol version {}", client.get_protocol_version());

    loop {
        let mut temp_max: f64 = 0.0;
        for chip in sensors.chip_iter(None) {
            for feature in chip.feature_iter() {
                 match feature.kind() {
                    Some(Temperature) => {
                        for sub_feature in feature.sub_feature_iter() {
                            if let Ok(TemperatureInput(value)) = sub_feature.value() {
                                if value > temp_max {
                                    temp_max = value;
                                }
                            }
                        }
                    },
                    Some(_) => (),
                    None => ()
                }
            }
        }
        update_color(&temp_max, &client).await;
        sleep(Duration::from_millis(500)).await;
    }
}

async fn update_color(temp: &f64, client: &OpenRGB<TcpStream>) {

    let mut r: u8 = BASE_C_VALUE;
    let mut g: u8 = BASE_C_VALUE;
    let b: u8 = 0;

    let temp_scale: f64 = (temp - LOWER_TEMP) / (UPPER_TEMP - LOWER_TEMP);

    if *temp > LOWER_TEMP {
        r = max((temp_scale * (MAX_C_VALUE - BASE_C_VALUE) as f64) as u8 + BASE_C_VALUE, MAX_C_VALUE);
        g = (1 / temp_scale * BASE_C_VALUE) as u8;
    }


    let num_controllers = match client.get_controller_count().await {
        Ok(res) => {
            println!("Found {} controllers", res);
            res
        },
        Err(error) => panic!("Couldn't read number of controllers from OpenRgb server: {:?}", error),
    };

    for controller_id in 0..num_controllers {
        if let Ok(controller) = client.get_controller(controller_id).await {
            let num_leds = controller.leds.len();
            match client.update_leds(controller_id, vec![Color{r, g, b}; num_leds]).await {
                Ok(()) => (),
                Err(error) => panic!("Couldn't set controller {}: {:?}", controller_id, error)
            };
        } else {
            println!("Couldn't fetch leds on controller {}", controller_id);
        }
        println!("Controller {} set to {}", controller_id, Color{r, g, b})
    }
}
