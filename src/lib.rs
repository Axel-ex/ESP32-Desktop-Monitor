use anyhow::Result;
use chrono::Local;
use chrono_tz::Europe::Lisbon;
use dht_sensor::*;
use embedded_graphics::{
    mono_font::{iso_8859_1::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use esp_idf_svc::hal::{
    delay::Ets,
    gpio::{Gpio2, Gpio21, Gpio22, Gpio27, InputOutput},
    gpio::{Output, PinDriver},
    i2c::{I2cConfig, I2cDriver, I2C0},
    peripherals::Peripherals,
    units::Hertz,
};
use esp_idf_svc::{
    sntp::{EspSntp, SyncStatus},
    wifi::{BlockingWifi, EspWifi},
};
use log::{info, warn};
use sh1106::{prelude::*, Builder};
pub mod wifi;
use crate::wifi::*;

const SCREEN_WIDTH: i32 = 128;
const SCREEN_HEIGHT: i32 = 64;
const TEXT_STYLE: MonoTextStyle<'static, BinaryColor> =
    MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

#[allow(dead_code)]
pub struct DeviceState {
    display: GraphicsMode<I2cInterface<I2cDriver<'static>>>,
    dht_pin: PinDriver<'static, Gpio27, InputOutput>,
    led: PinDriver<'static, Gpio2, Output>,
    wifi: BlockingWifi<EspWifi<'static>>,
    sntp: EspSntp<'static>,
    signal_tracker: WifiSignalTracker,
}

///Interface with the
impl DeviceState {
    pub fn new() -> Result<Self> {
        let p = Peripherals::take()?;

        let display = Self::init_display(p.i2c0, p.pins.gpio21, p.pins.gpio22)?;

        let mut dht_pin = PinDriver::input_output_od(p.pins.gpio27)?;
        dht_pin.set_high().unwrap();

        let led = PinDriver::output(p.pins.gpio2)?;

        let mut wifi = wifi_init(p.modem)?;
        wifi_connect(&mut wifi)?;

        let sntp = Self::sync_sntp()?;

        let signal_tracker = WifiSignalTracker::default();

        Ok(DeviceState {
            display,
            dht_pin,
            led,
            wifi,
            sntp,
            signal_tracker,
        })
    }

    ///Initialize display using
    fn init_display(
        i2c0: I2C0,
        sda: Gpio21,
        scl: Gpio22,
    ) -> Result<GraphicsMode<I2cInterface<I2cDriver<'static>>>> {
        let config = I2cConfig::new().baudrate(Hertz(100_000));
        let i2c = I2cDriver::new(i2c0, sda, scl, &config)?;
        let mut display: GraphicsMode<I2cInterface<I2cDriver>> =
            Builder::new().connect_i2c(i2c).into();
        display.init().expect("fail to init display");

        Ok(display)
    }

    /// start Server ntp (network time protocol) and synchronize the system time.
    fn sync_sntp() -> Result<EspSntp<'static>> {
        let sntp = EspSntp::new_default()?;
        info!("SNTP initialized, waiting for status!");

        while sntp.get_sync_status() != SyncStatus::Completed {}
        info!("SNTP status received!");

        Ok(sntp)
    }

    pub fn clear_display(&mut self) {
        self.display.clear();
    }

    pub fn flush_display(&mut self) {
        self.display
            .flush()
            .expect("Error occured while flushing the display");
    }

    pub fn display_layout(&mut self) {
        Rectangle::with_corners(
            Point::new(0, 0),
            Point::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2),
        )
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut self.display)
        .unwrap();

        Rectangle::with_corners(
            Point::new(SCREEN_WIDTH / 2, 0),
            Point::new(SCREEN_WIDTH - 1, SCREEN_HEIGHT / 2),
        )
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut self.display)
        .unwrap();

        Rectangle::with_corners(
            Point::new(0, SCREEN_HEIGHT / 2),
            Point::new(SCREEN_WIDTH - 1, SCREEN_HEIGHT - 1),
        )
        .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
        .draw(&mut self.display)
        .unwrap();
    }

    pub fn display_time(&mut self) {
        // Get the current time in the desired time zone
        let time = Local::now().with_timezone(&Lisbon).time().to_string();
        info!("current time: {:?}", time);

        // Draw the time on the display without seconds
        Text::new(&time[..5], Point::new(12, 17), TEXT_STYLE)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn display_temperature(&mut self) {
        let mut delay_provider = Ets;

        match dht22::read(&mut delay_provider, &mut self.dht_pin) {
            Ok(reading) => {
                info!(
                    "{}°C, {}%",
                    reading.temperature - 4f32,
                    reading.relative_humidity
                );

                Text::new(
                    format!("{}°C", reading.temperature - 4f32).as_str(),
                    Point { x: 78, y: 11 },
                    TEXT_STYLE,
                )
                .draw(&mut self.display)
                .unwrap();

                Text::new(
                    format!("{}%", reading.relative_humidity).as_str(),
                    Point { x: 78, y: 25 },
                    TEXT_STYLE,
                )
                .draw(&mut self.display)
                .unwrap();
            }
            Err(e) => info!("{:#?}", e),
        }
    }

    pub fn display_wifi_info(&mut self) {
        let scan_result = self.wifi.wifi_mut().scan();
        let network = WIFI_SSID;

        match scan_result {
            Ok(access_points) => {
                // Filter to find the access point with SSID "MEO-BD8310"
                if let Some(net) = access_points.iter().find(|ap| ap.ssid == network) {
                    info!("Found {network}");
                    info!("Signal Strength: {} dBm", net.signal_strength);

                    // Take the absolute value
                    let abs_signal_strength = net.signal_strength.abs();

                    // Scale the absolute value to fit within the range [32, 64]
                    let scaled_strength = ((abs_signal_strength - 20) as f32 * (64 - 32) as f32
                        / (80 - 20) as f32
                        + 32.0) as u32;

                    //get the current x pos and add to the vector
                    let x = self.signal_tracker.get_x_pos();
                    self.signal_tracker.add_point(x, scaled_strength);

                    //print wifi signal
                    for point in self.signal_tracker.get_points() {
                        self.display.set_pixel(point.x as u32, point.y as u32, 1);
                    }
                    let avg = self.signal_tracker.get_average_strength().to_string();
                    let avg = format!("-{}", avg);

                    Text::new(
                        avg.as_str(),
                        Point::new(SCREEN_WIDTH - 20, SCREEN_HEIGHT - 20),
                        TEXT_STYLE,
                    )
                    .draw(&mut self.display)
                    .unwrap();

                    self.signal_tracker.increment_x_pos();
                } else {
                    warn!("{network} not found.");
                }
            }
            Err(e) => {
                warn!("Failed to scan WiFi networks: {:?}", e);
            }
        }
    }

    pub fn led_on(&mut self) {
        self.led.set_high().unwrap();
    }

    pub fn led_off(&mut self) {
        self.led.set_low().unwrap();
    }
}
