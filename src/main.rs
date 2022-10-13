#![feature(iter_advance_by)]
use esp_idf_sys::{time_t, esp_light_sleep_start, esp_sleep_enable_timer_wakeup};
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

use esp_idf_hal::delay::{self, Ets};
use esp_idf_hal::prelude::Peripherals;

use embedded_graphics::mono_font::{MonoTextStyleBuilder, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
};

use esp_idf_hal::spi;
use esp_idf_hal::spi::*;

use esp_idf_hal::prelude::FromValueType;

use std::ptr;
use std::sync::Arc;
use std::time::Duration;

use time::format_description;
use time::macros::offset;
use time::OffsetDateTime;

use anyhow::bail;
use log::*;

use soup::prelude::*;

use ili9341::{self, Orientation};

use display_interface_spi::SPIInterfaceNoCS;

use epd_waveshare::{
    color::*,
    epd2in9_v2::{Display2in9, Epd2in9},
    graphics::DisplayRotation,
    prelude::*,
};

const SSID: &str = "test";
const PASS: &str = "qwerqwer";

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();
    info!("Start main");

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
    let _wifi = wifi(
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

    // GPIOs for real HW wrover-kit
    let mut spi = peripherals.spi2;
    // let backlight = pins.gpio5;
    // let dc = pins.gpio21;
    // let rst = pins.gpio18;
    // let sclk = pins.gpio19;
    // let miso = pins.gpio25;
    // let mosi = pins.gpio23;
    // let cs = pins.gpio22;
    // let busy = pins.gpio4;

    let cs = pins.gpio22;
    let dc = pins.gpio21;
    let sclk = pins.gpio19;
    let rst = pins.gpio18;
    let busy = pins.gpio5.into_input()?;
    let mosi = pins.gpio23;
    let miso = pins.gpio16;

    //GPIOs for Wokwi simulation
    // let spi = peripherals.spi2;
    // let backlight = pins.gpio5;
    // let dc = pins.gpio21;
    // let rst = pins.gpio18;
    // let sclk = pins.gpio19;
    // let miso = pins.gpio25;
    // let mosi = pins.gpio23;
    // let cs = pins.gpio22;

    // display
    let config = <spi::config::Config as Default>::default().baudrate((26_000_000).into());

    //let mut backlight = backlight.into_output()?;
    //backlight.set_low()?;

    //https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/peripherals/spi_master.html
    //using real HW wrover-kit, please, use spi::Master::<spi::SPI2, _, _, _, _>::new instead of spi::Master::<spi::SPI3, _, _, _, _>::new
    // let di = SPIInterfaceNoCS::new(
    //     spi::Master::<spi::SPI2, _, _, _, _>::new(
    //         spi,
    //         spi::Pins {
    //             sclk,
    //             sdo: mosi,
    //             sdi: Some(miso),
    //             cs: Some(cs),
    //         },
    //         config,
    //     )?,
    //     dc.into_output()?,
    // );

    let reset = rst.into_output()?;
    //let backlight = backlight.into_output()?;

    // let mut display = ili9341::Ili9341::new(
    //     di,
    //     reset,
    //     &mut delay::Ets,
    //     Orientation::Portrait,
    //     ili9341::DisplaySize240x320,
    // )
    // .map_err(|_| anyhow::anyhow!("Display"))?;

    let config = config::Config::new().baudrate(26u32.MHz().into());
    // let mut spi =
    //     SPI2::new(spi, sclk, mosi, Some(busy), Some(cs), &config)?;

    let mut delay: Ets = Ets::into(Ets);

    let config = <spi::config::Config as Default>::default().baudrate(26.MHz().into());
    let mut spi = spi::Master::<spi::SPI2, _, _, _, _>::new(
        spi,
        spi::Pins {
            sclk,
            sdo: mosi,
            sdi: Some(miso),
            cs: Some(pins.gpio15),
        },
        config,
    )?;

    let mut epd = Epd2in9::new(
        &mut spi,
        cs.into_output()?,
        busy.into_input()?,
        dc.into_output()?,
        reset.into_output()?,
        &mut delay).unwrap();

    let mut display = Display2in9::default();


    let mut display = Display2in9::default();
    display.set_rotation(DisplayRotation::Rotate90);
    println!("Color: {:?}, Is busy {:?}", epd.background_color(), epd.is_busy());

    display.set_rotation(DisplayRotation::Rotate90);

    draw_text_epaper(&mut display, " Hello Rust from ESP32dfg! ", 15, 50);
    epd.update_and_display_frame(&mut spi, display.buffer(), &mut delay).expect("Frame cannot be cleared and updated!");

    
    /* 
    unsafe {
        esp_sleep_enable_timer_wakeup(60_000);
        // match t {
        //     Some(x) => { info!("Dobre je"); }
        //     _ => { info!("Zle je"); }
        // }
        esp_light_sleep_start();
    }
*/
    // draw_text_lcd(
    //     &mut display,
    //     &"".to_string(),
    //     &"Initialization...".to_string(),
    // );

    let url = String::from(
        "https://idos.idnes.cz/en/brno/odjezdy/vysledky/?f=Technologick%C3%BD%20park&fc=302003",
    );

    let mut text_to_dislay: Vec<String> = Vec::new();

    loop {
        info!("About to fetch content0 from {}", url);

        let mut client = EspHttpClient::new_default()?;

        info!("About to fetch content1 from {}", url);

        let response = client.get(&url)?.submit()?;

        info!("About to fetch content2 from {}", url);

        let body: Result<Vec<u8>, _> = Bytes::<_, 8>::new(response.reader()).collect();

        info!("About to fetch content3 from {}", url);

        let body = body?;
        let str = String::from_utf8(body)?;

        info!("About to fetch content4 from {}", url);

        let soup = Soup::new(&str);

        // display

        info!("About to initialize the ILI9341 SPI LED driver",);

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
                        "[day].[month padding:none].[year] [hour padding:none]:[minute]:[second] [offset_hour \
                        sign:mandatory]:[offset_minute]:[offset_second]",
                    )?;

                    let timer: *mut time_t = ptr::null_mut();
                    text_to_dislay.push(format!(
                        "{}:\n{}\n",
                        &direction.unwrap(),
                        &OffsetDateTime::parse(&time, &format)?.time().to_string()
                    ));
                } else {
                    continue;
                }
            }
        }
        //println!("text_to_display: {:?}", &text_to_dislay);
        //let s: String = text_to_dislay.to_owned().into_iter().take(6).collect();
        unsafe {
            let timer: *mut time_t = ptr::null_mut();
            //let acutal_time_timestamp = esp_idf_sys::time(timer);

            // let time = OffsetDateTime::from_unix_timestamp(esp_idf_sys::time(timer) as i64)?
            //     .to_offset(offset!(+2))
            //     .time();

            //format!("Actual time: {}\n", &time.to_string())
            draw_text_lcd(
                &mut display,
                &text_to_dislay.to_owned().into_iter().take(7).collect(),
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
        //thread::sleep(Duration::from_secs(10));
        info!("About to sleep");
        drop(wifi);
        info!("Wifi stopped");
        unsafe {
            esp_sleep_enable_timer_wakeup(60_000);
            // match t {
            //     Some(x) => { info!("Dobre je"); }
            //     _ => { info!("Zle je"); }
            // }
            esp_light_sleep_start();
        }

        // unsafe {
        //     esp_deep_sleep(10_000_000);
        // }

        info!("About to wake up");
        // let _wifi = wifi(
        //     netif_stack.clone(),
        //     sys_loop_stack.clone(),
        //     default_nvs.clone(),
        // )?;
    }
}

fn draw_text_epaper(display: &mut Display2in9, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
        .text_color(Black)
        .background_color(White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}

/*
scan all accessible access points
find "ours" access point with SSID
configure WiFi with SSID, PASS, channel,...
get status -> make connection
*/
#[allow(dead_code)]
fn wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> anyhow::Result<Box<EspWifi>> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASS.into(),
        auth_method: AuthMethod::None,
        ..Default::default()
    }))?;

    println!("Wifi configuration set, about to get status");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    info!("to get status");
    let status = wifi.get_status();

    info!("got status)");
    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(
            _ip_settings,
        ))),
        _,
    ) = status
    {
        println!("Wifi connected");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

#[cfg(feature = "esp32_wrover_ili9341")]
#[allow(dead_code)]
fn draw_text_lcd<D>(display: &mut D, text: &String, time: &String) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    //let rect = Rectangle::new(display.bounding_box().top_left, display.bounding_box().size);

    display.clear(Rgb565::BLACK.into())?;
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
