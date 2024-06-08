use anyhow::{Error, Result};
use chrono::{DateTime, Utc};
use dht_sensor::*;
use embedded_graphics::{
    mono_font::{iso_8859_1::FONT_6X10, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use esp_idf_svc::hal::gpio::{Gpio2, Gpio21, Gpio22, Gpio27, InputOutput};
use esp_idf_svc::hal::{
    delay::Ets,
    gpio::{Output, PinDriver},
    i2c::{I2cConfig, I2cDriver, I2C0},
    peripherals::Peripherals,
    units::Hertz,
};
use esp_idf_svc::sntp::{EspSntp, SyncStatus};
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log;
use sh1106::{prelude::*, Builder};
use std::time::{self, UNIX_EPOCH};

pub mod wifi;
use crate::wifi::{wifi_connect, wifi_init};

pub struct DeviceState<'a> {
    display: GraphicsMode<I2cInterface<I2cDriver<'a>>>,
    dht_pin: PinDriver<'a, Gpio27, InputOutput>,
    led: PinDriver<'a, Gpio2, Output>,
    wifi: BlockingWifi<EspWifi<'a>>,
    sntp: EspSntp<'a>,
}

impl<'a> DeviceState<'a> {
    pub fn new() -> Result<Self> {
        let p = Peripherals::take().unwrap();

        // let sda = p.pins.gpio21;
        // let scl = p.pins.gpio22;
        // let config = I2cConfig::new().baudrate(Hertz(100_000)).into();
        // let i2c = I2cDriver::new(p.i2c0, sda, scl, &config)?;
        // let mut display: GraphicsMode<I2cInterface<I2cDriver>> =
        //     Builder::new().connect_i2c(i2c).into();
        // display.init().unwrap();
        let display = Self::init_display(p.i2c0, p.pins.gpio21, p.pins.gpio22)?;

        let mut dht_pin = PinDriver::input_output_od(p.pins.gpio27)?;
        dht_pin.set_high().unwrap();

        let led = PinDriver::output(p.pins.gpio2)?;

        let mut wifi = wifi_init(p.modem)?;
        wifi_connect(&mut wifi)?;

        //sntp
        let sntp = EspSntp::new_default().unwrap();
        log::info!("SNTP initialized, waiting for status!");

        while sntp.get_sync_status() != SyncStatus::Completed {}

        log::info!("SNTP status received!");

        Ok(DeviceState {
            display,
            dht_pin,
            led,
            wifi,
            sntp,
        })
    }

    fn init_display(
        i2c0: I2C0,
        sda: Gpio21,
        scl: Gpio22,
    ) -> Result<GraphicsMode<I2cInterface<I2cDriver<'a>>>> {
        let config = I2cConfig::new().baudrate(Hertz(100_000)).into();
        let i2c = I2cDriver::new(i2c0, sda, scl, &config)?;
        let mut display: GraphicsMode<I2cInterface<I2cDriver>> =
            Builder::new().connect_i2c(i2c).into();
        display.init().expect("fail to init display");

        Ok(display)
    }

    pub fn clear_display(&mut self) {
        self.display.clear();
    }

    pub fn flush_display(&mut self) {
        self.display.flush().unwrap();
    }

    pub fn display_layout(&mut self) {
        Rectangle::with_corners(Point::new(0, 0), Point::new(64, 32))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();

        Rectangle::with_corners(Point::new(64, 0), Point::new(64 + 63, 32))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();

        Rectangle::with_corners(Point::new(0, 32), Point::new(127, 32 + 31))
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn display_date(&mut self) {
        let text_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        let now = time::SystemTime::now().duration_since(UNIX_EPOCH);
        match now {
            Ok(now) => {
                let datetime = DateTime::<Utc>::from(UNIX_EPOCH + now);
                log::info!("current time: {}", datetime);
            }
            Err(e) => log::info!("error occurred getting time: {:?}", e),
        }
        Text::new("Tu 22:24", Point { x: 8, y: 17 }, text_style)
            .draw(&mut self.display)
            .unwrap();
    }

    pub fn display_temperature(&mut self) {
        let mut delay = Ets;
        let temp_style = MonoTextStyle::new(&FONT_6X10, BinaryColor::On);

        match dht22::read(&mut delay, &mut self.dht_pin) {
            Ok(reading) => {
                log::info!("{}°C, {}%", reading.temperature, reading.relative_humidity);

                Text::new(
                    format!("{}°C", reading.temperature - 4f32).as_str(),
                    Point { x: 78, y: 11 },
                    temp_style,
                )
                .draw(&mut self.display)
                .unwrap();

                Text::new(
                    format!("{}%", reading.relative_humidity).as_str(),
                    Point { x: 78, y: 25 },
                    temp_style,
                )
                .draw(&mut self.display)
                .unwrap();
            }
            Err(e) => println!("{:#?}", e),
        }
    }

    pub fn led_on(&mut self) {
        self.led.set_high().unwrap();
    }

    pub fn led_off(&mut self) {
        self.led.set_low().unwrap();
    }
}
