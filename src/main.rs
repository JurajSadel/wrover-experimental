#![feature(iter_advance_by)]
use embedded_graphics::mono_font::iso_8859_2;
use esp_idf_sys::time_t;
use esp_idf_sys::{self as _};

use embedded_svc::http::client::*;
use embedded_svc::io::Bytes;
use embedded_svc::wifi::*;

use esp_idf_svc::http::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sntp;
use esp_idf_svc::sntp::SyncStatus;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::delay;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::spi;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};

use ili9341::Scroller;

use regex::Regex;
use time::format_description::modifier::Hour;

use std::ptr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use time::format_description;
use time::macros::offset;
use time::OffsetDateTime;

use anyhow::bail;
use log::*;

use soup::prelude::*;

use ili9341::{self, Orientation};

use display_interface_spi::SPIInterfaceNoCS;

const SSID: &str = "test"; //env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = "qwerqwer"; //env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    /*
    1 initialization(wifi, display, peripherals,....)
    2 loop
        3 fetch data from url
        4 reload vector (5 lines)
        5 display new results
        6 sleep for 45-60 secs
    */

    // wifi part
    #[allow(unused)]
    let netif_stack = Arc::new(EspNetifStack::new()?);
    #[allow(unused)]
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    #[allow(unused)]
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let wifi = wifi(
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;

    let sntp = sntp::EspSntp::new_default()?;
    info!("SNTP initialized");

    while sntp.get_sync_status() != SyncStatus::Completed {}

    info!("Waiting for SyncStatus done");

    // peripherals
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let spi = peripherals.spi2;
    let backlight = pins.gpio5;
    let dc = pins.gpio21;
    let rst = pins.gpio18;
    let sclk = pins.gpio19;
    let miso = pins.gpio25;
    let mosi = pins.gpio23;
    let cs = pins.gpio22;

    // display
    let config = <spi::config::Config as Default>::default().baudrate((26_000_000).into());

    //let mut backlight = backlight.into_output()?;
    //backlight.set_low()?;

    //https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/peripherals/spi_master.html
    let di = SPIInterfaceNoCS::new(
        spi::Master::<spi::SPI2, _, _, _, _>::new(
            spi,
            spi::Pins {
                sclk,
                sdo: mosi,
                sdi: Some(miso),
                cs: Some(cs),
            },
            config,
        )?,
        dc.into_output()?,
    );

    let reset = rst.into_output()?;
    let backlight = backlight.into_output()?;

    let mut display = ili9341::Ili9341::new(
        di,
        reset,
        &mut delay::Ets,
        Orientation::PortraitFlipped,
        ili9341::DisplaySize240x320,
    )
    .map_err(|_| anyhow::anyhow!("Display"))?;

    led_draw(
        &mut display,
        &"".to_string(),
        &"Initialization in progress...".to_string(),
    );

    let url = String::from(
        "https://idos.idnes.cz/en/brno/odjezdy/vysledky/?f=Technologick%C3%BD%20park&fc=302003",
    );

    let mut text_to_dislay: Vec<String> = Vec::new();

    loop {
        info!("About to fetch content from {}", url);

        let mut client = EspHttpClient::new_default()?;

        info!("after new default");

        let response = client.get(&url)?.submit()?;

        info!("after get");

        let body: Result<Vec<u8>, _> = Bytes::<_, 8>::new(response.reader()).collect();

        let body = body?;
        let str = String::from_utf8(body)?;
        //let str = String::from_utf8_lossy(&body)?;

        //let document = Html::parse_document(&body);
        //let document = Dom::parse(&String::from_utf8_lossy(&body))?;

        let soup = Soup::new(&str);
        //println!("Soup {:?}", soup);
        //let times = soup.tag("table").find_all().nth(n);

        // display

        info!("About to initialize the ILI9341 SPI LED driver",);

        //let mut scroller = display.configure_vertical_scroll(20, 5).map_err(|_| anyhow::anyhow!("Display"))?;
        // display.scroll_vertically(scroller, 5);

        if text_to_dislay.is_empty() {
            let links = soup.tag("tr").find_all();

            for link in links {
                let time = link.get("data-datetime");
                let direction = link.get("data-stationname");

                if direction.is_some() {
                    let mut time = time.unwrap();

                    //add offset
                    time.push_str(" +02:00:00");
                    let format = format_description::parse(
                        "[day].[month padding:none].[year] [hour padding:zero]:[minute]:[second] [offset_hour \
                        sign:mandatory]:[offset_minute]:[offset_second]",
                    )?;

                    unsafe {
                        let timer: *mut time_t = ptr::null_mut();
                        text_to_dislay.push(format!(
                            "{}:\n{}\n",
                            &direction.unwrap(),
                            &OffsetDateTime::parse(&time, &format)?.time().to_string()
                        ));
                    }
                } else {
                    continue;
                }
            }
        }
        println!("text_to_display: {:?}", &text_to_dislay);
        //let s: String = text_to_dislay.to_owned().into_iter().take(6).collect();
        unsafe {
            let timer: *mut time_t = ptr::null_mut();
            //let acutal_time_timestamp = esp_idf_sys::time(timer);

            // let time = OffsetDateTime::from_unix_timestamp(esp_idf_sys::time(timer) as i64)?
            //     .to_offset(offset!(+2))
            //     .time();

            //format!("Actual time: {}\n", &time.to_string())
            led_draw(
                &mut display,
                &text_to_dislay.to_owned().into_iter().take(6).collect(),
                &format!(
                    "Actual time: {}\n",
                    &OffsetDateTime::from_unix_timestamp(esp_idf_sys::time(timer) as i64)?
                        .to_offset(offset!(+2))
                        .time()
                        .to_string()
                ),
            );
        }
        text_to_dislay.clear();
        thread::sleep(Duration::from_secs(10));
    }
    info!("About to sleep");
    //thread::sleep(Duration::from_millis(1000));

    drop(wifi);
    info!("Wifi stopped");

    Ok(())
}

#[allow(dead_code)]
fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> anyhow::Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(
            _ip_settings,
        ))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

#[allow(dead_code)]
fn led_draw<D>(display: &mut D, text: &String, time: &String) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    //let rect = Rectangle::new(display.bounding_box().top_left, display.bounding_box().size);

    //display.clear(Rgb565::BLACK.into())?;
    //display.fill_solid(&rect, Rgb565::GREEN.into());

    Rectangle::new(Point::zero(), Size::new(300, 20)).into_styled(
        TextBoxStyleBuilder::new()
            .height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Justified)
            .paragraph_spacing(3)
            .build(),
    );
    //.draw(display)?;

    Text::with_alignment(
        &time,
        Point::new(0, 15),
        MonoTextStyle::new(
            &embedded_graphics::mono_font::iso_8859_2::FONT_10X20,
            Rgb565::WHITE.into(),
        ),
        Alignment::Left,
    )
    .draw(display)?;

    Rectangle::new(Point::zero(), Size::new(300, 300)).into_styled(
        TextBoxStyleBuilder::new()
            //.height_mode(HeightMode::FitToText)
            .alignment(HorizontalAlignment::Justified)
            .paragraph_spacing(3)
            .build(),
    );
    //.draw(display)?;

    Text::with_alignment(
        &text,
        Point::new(0, 30),
        MonoTextStyle::new(
            &embedded_graphics::mono_font::iso_8859_2::FONT_10X20,
            Rgb565::WHITE.into(),
        ),
        Alignment::Left,
    )
    .draw(display)?;

    info!("Displaying done");

    Ok(())
}
