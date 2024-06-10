use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::log::EspLogger;
use rust_clock::*;

fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let mut device_state = DeviceState::new().unwrap_or_else(|e| {
        println!("{:?}", e);
        panic!("Failed to initialize device state");
    });

    loop {
        device_state.led_on();
        device_state.clear_display();
        device_state.display_layout();
        device_state.display_time();
        device_state.display_temperature();
        device_state.display_wifi_info();
        device_state.flush_display();
        device_state.led_off();
        FreeRtos::delay_ms(1000);
    }
}
