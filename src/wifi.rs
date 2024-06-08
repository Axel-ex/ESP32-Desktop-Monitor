use anyhow::{Error, Result};
use embedded_svc::wifi::{ClientConfiguration, Configuration};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use heapless::String;
use log::info;

pub fn wifi_init<'a>(modem: Modem) -> Result<BlockingWifi<EspWifi<'a>>, Error> {
    let sys_loop = EspSystemEventLoop::take().expect("fail taking eventloop");
    let nvs = EspDefaultNvsPartition::take().expect("fail taking nvs");

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sys_loop.clone(), Some(nvs)).expect(" no nvs"),
        sys_loop,
    )?;

    Ok(wifi)
}

pub fn wifi_connect(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<()> {
    let wifi_config: Configuration = Configuration::Client(ClientConfiguration {
        ssid: String::try_from("MEO-BD8310").unwrap(),
        bssid: None,
        password: String::try_from("9f24731014").unwrap(),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_config)?;
    wifi.start()?;
    info!("Starting wifi");

    wifi.connect()?;
    info!("Connecting.....");

    wifi.wait_netif_up()?;
    info!("Netif up");

    Ok(())
}
