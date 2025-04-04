mod private {
    use crate::core::{Result, SmartPotError, esp::board::Board};
    use esp_idf_hal::gpio::OutputPin;
    use esp_idf_svc::http::server::EspHttpServer;
    use log::{error, info};
    use smart_pot_core::TemperatureUnits;
    use std::sync::{Arc, Mutex};

    pub fn server_init<T: OutputPin>(
        board: Arc<Mutex<Board<T>>>,
    ) -> Result<EspHttpServer<'static>> {
        let mut server = EspHttpServer::new(&Default::default())?;

        // GET / — telemetry JSON
        {
            let board = Arc::clone(&board);
            server.fn_handler("/telemetry", embedded_svc::http::Method::Get, move |req| {
                let mut res = match req.into_ok_response() {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Response error: {:?}", e);
                        return Err(SmartPotError::EspIoError(e));
                    }
                };

                let mut board = match board.lock() {
                    Ok(b) => b,
                    Err(_) => return Err(SmartPotError::MutexError),
                };
                let telemetry = board.get_telemetry();

                match serde_json::to_string(&telemetry) {
                    Ok(telemetry) => {
                        res.write(telemetry.as_bytes())?;
                        Ok(())
                    }
                    Err(err) => Err(SmartPotError::SerializationError(err)),
                }
            })?;
        }

        // POST /light/on
        {
            let board = Arc::clone(&board);
            server.fn_handler(
                "/direct-method/light/on",
                embedded_svc::http::Method::Put,
                move |_| {
                    if let Ok(mut board) = board.lock() {
                        board.light_on()?;
                    } else {
                        error!("Board lock failed on /light/on");
                        return Err(SmartPotError::MutexError);
                    }
                    info!("Light on.");
                    Ok::<(), SmartPotError>(())
                },
            )?;
        }

        // POST /light/off
        {
            let board = Arc::clone(&board);
            server.fn_handler(
                "/direct-method/light/off",
                embedded_svc::http::Method::Put,
                move |_| {
                    if let Ok(mut board) = board.lock() {
                        board.light_off()?;
                    } else {
                        return Err(SmartPotError::MutexError);
                    }
                    info!("Light off.");
                    Ok(())
                },
            )?;
        }

        // POST /c2d/far
        {
            let board = Arc::clone(&board);
            server.fn_handler("/c2d/far", embedded_svc::http::Method::Put, move |_| {
                if let Ok(mut board) = board.lock() {
                    board.set_temperature_units(TemperatureUnits::Fahrenheit);
                } else {
                    return Err(SmartPotError::MutexError);
                }
                info!("Temperature metrics set to fahrenheit");

                Ok::<(), SmartPotError>(())
            })?;
        }

        // POST /c2d/cel
        {
            let board = Arc::clone(&board);
            server.fn_handler("/c2d/cel", embedded_svc::http::Method::Put, move |_| {
                if let Ok(mut board) = board.lock() {
                    board.set_temperature_units(TemperatureUnits::Celsius);
                } else {
                    return Err(SmartPotError::MutexError);
                }
                info!("Temperature metrics set to celsius");
                Ok::<(), SmartPotError>(())
            })?;
        }

        Ok(server)
    }
}

crate::mod_interface! {
    orphan use {
        server_init
    };
}
