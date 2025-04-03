//!
//! Wi-Fi Connection Module for SmartPot
//!

mod private {
    use crate::core::Result;
    use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
    use esp_idf_hal::peripheral::Peripheral;
    use esp_idf_svc::{
        eventloop::EspSystemEventLoop,
        nvs::{EspNvsPartition, NvsDefault},
        ping::EspPing,
        timer::{EspTimerService, Task},
        wifi::{AsyncWifi, EspWifi},
    };
    use heapless::String;
    use std::str::FromStr;

    /// Connects to a Wi-Fi network with the specified SSID and password.
    ///
    /// This asynchronous function establishes a Wi-Fi connection using the provided SSID and password.
    ///
    /// # Parameters:
    /// - `ssid`: The SSID of the Wi-Fi network to connect to.
    /// - `password`: The password for the Wi-Fi network.
    /// - `modem`: The modem to use for the Wi-Fi connection.
    /// - `sysloop`: The system event loop for handling events.
    /// - `nvs`: An optional NVS partition to store Wi-Fi settings.
    /// - `timer_service`: The timer service for handling timed events.
    ///
    /// # Returns:
    /// - `Result<AsyncWifi<EspWifi<'static>>>`: The function returns an instance of `AsyncWifi` wrapped around
    ///   an `EspWifi` object, which can be used for further Wi-Fi operations.
    ///
    /// # Errors:
    /// The function may return errors if there is an issue with the Wi-Fi connection or related configurations.
    pub async fn wifi<'a>(
        ssid: &str,
        password: &str,
        modem: impl Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
        sysloop: &'a EspSystemEventLoop,
        nvs: Option<EspNvsPartition<NvsDefault>>,
        timer_service: &'a EspTimerService<Task>,
    ) -> Result<AsyncWifi<EspWifi<'static>>> {
        let mut wifi = AsyncWifi::wrap(
            EspWifi::new(modem, sysloop.clone(), nvs)?,
            sysloop.clone(),
            timer_service.clone(),
        )?;

        connect(&mut wifi, ssid, password).await?;

        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

        log::info!("Wifi DHCP info: {:?}", ip_info);

        EspPing::default().ping(
            ip_info.subnet.gateway,
            &esp_idf_svc::ping::Configuration::default(),
        )?;
        Ok(wifi)
    }

    /// Internal helper function to configure and connect to a Wi-Fi network.
    async fn connect(wifi: &mut AsyncWifi<EspWifi<'_>>, ssid: &str, password: &str) -> Result<()> {
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: String::<32>::from_str(ssid).expect("Invalid wifi SSID"),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: String::from_str(password).expect("Invalid wifi password"),
            channel: None,
            pmf_cfg: esp_idf_svc::wifi::PmfConfiguration::NotCapable,
            scan_method: esp_idf_svc::wifi::ScanMethod::FastScan,
        });

        wifi.set_configuration(&wifi_configuration)?;

        wifi.start().await?;
        wifi.connect().await?;
        wifi.wait_netif_up().await?;

        log::info!("Wifi connected");

        Ok(())
    }
}

crate::mod_interface! {
    orphan use {
        wifi
    };
}
