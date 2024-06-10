use anyhow::Result;
use embedded_graphics::prelude::*;
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use heapless::String;
use log::info;

const SCREEN_WIDTH: u32 = 128;
pub const WIFI_SSID: &str = "MEO-BD8310";
const WIFI_PASSWORD: &str = "9f24731014";

/// Tracks the wifi signal by storing the results of the wifi scan to further draw them on screen.
/// keep also track of the current x pos on screen and wraps it to SCREEN_WIDTH.
pub struct WifiSignalTracker {
    points: Vec<Point>,
    max_points: usize,
    curr_x_pos: u32,
}

impl Default for WifiSignalTracker {
    fn default() -> Self {
        Self::new(SCREEN_WIDTH as usize)
    }
}

impl WifiSignalTracker {
    ///Create a new instance of WifiSignalTracker with a mazimum size = SCREEN_WIDTH
    pub fn new(max_points: usize) -> Self {
        Self {
            points: Vec::with_capacity(max_points),
            curr_x_pos: 0,
            max_points,
        }
    }

    ///add new data_point to the vector
    pub fn add_point(&mut self, x: u32, y: u32) {
        if self.points.len() >= self.max_points {
            self.points.remove(0); // Remove the oldest point
        }
        self.points.push(Point::new(x as i32, y as i32)); // Add the new point
    }

    //Get the underlying vector of points
    pub fn get_points(&self) -> &[Point] {
        &self.points
    }

    ///increment the current x position and wraps it to SCREEN_WIDTH, reseting it to 0 when it
    ///reaches the end of the screen.
    pub fn increment_x_pos(&mut self) {
        self.curr_x_pos = (self.curr_x_pos + 1) % SCREEN_WIDTH;
    }

    pub fn get_x_pos(&mut self) -> u32 {
        self.curr_x_pos
    }

    ///Compute the average signal strength (y) of the vector of points
    pub fn get_average_strength(&mut self) -> i32 {
        if self.points.is_empty() {
            return 0i32;
        }
        let total_strength: i32 = self.points.iter().map(|&point| point.y).sum();

        total_strength / (self.points.len() as i32)
    }

    pub fn print_points(&self) {
        for point in &self.points {
            println!("Point - x: {}, y: {}", point.x, point.y);
        }
    }
}

/// Initialize the WiFi network.
pub fn wifi_init<'a>(modem: Modem) -> Result<BlockingWifi<EspWifi<'a>>> {
    let sys_loop = EspSystemEventLoop::take().expect("fail taking eventloop");
    let nvs = EspDefaultNvsPartition::take().expect("fail taking nvs");

    let wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;

    Ok(wifi)
}

///Connect to the wifi
pub fn wifi_connect(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<()> {
    let wifi_config: Configuration = Configuration::Client(ClientConfiguration {
        ssid: String::try_from(WIFI_SSID).expect("Invalid WIFI SSID"),
        bssid: None,
        password: String::try_from(WIFI_PASSWORD).expect("Invalid WiFi password"),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_config)?;
    info!("Starting wifi");
    wifi.start()?;

    info!("Connecting.....");
    wifi.connect()?;

    wifi.wait_netif_up()?;
    info!("Netif up");

    Ok(())
}
