use embedded_graphics::mono_font::iso_8859_2;
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_svc::http::client::*;
use embedded_svc::io::Bytes;
use embedded_svc::wifi::*;

use esp_idf_svc::http::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::spi;
use esp_idf_hal::delay;
use esp_idf_hal::prelude::Peripherals;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

//use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use regex::Regex;

use anyhow::bail;
//use anyhow::Result;
use log::*;

//use soup::prelude::*;

//use html_parser::{Dom, Element, Node};
use soup::prelude::*;

use ili9341::{self, Orientation};

use display_interface_spi::SPIInterfaceNoCS;
//use html_parser::Node::Element;

// use select::document::Document;
// use select::predicate::{Attr, Class, Name, Predicate};

// use scraper::Html;


const SSID: &str = "test";//env!("RUST_ESP32_STD_DEMO_WIFI_SSID");
const PASS: &str = "qwerqwer";//env!("RUST_ESP32_STD_DEMO_WIFI_PASS");

fn main() -> anyhow::Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    esp_idf_svc::log::EspLogger::initialize_default();

    //wifi part
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

    let url = String::from("https://idos.idnes.cz/en/brno/odjezdy/vysledky/?f=Technologick%C3%BD%20park&fc=302003");

    //let mut client = EspHttpClient::new_default()?;

    //let response = client.get(&url)?.submit()?;
    //let body: Result<Vec<u8>, _> = Bytes::<_, 64>::new(response.reader()).collect();

    //let document = Html::parse_document(html);
    //println!("Doc {:?}", document);
    //let document = Document::from(html);
    // println!("# Menu");
    // for node in document.nodes {
    //     println!("Data {:?}", node.data);
    //     println!("F.CH {:?}", node.first_child);
    //     //println!("select {:?}", node.find(Class("departures-table__cell")));
    // }
    //for node in document.find(Class("departures-table__cell")) {
        //let question = node.find(Class("question-hyperlink")).next().unwrap();
        //let votes = node.find(Class("vote-count-post")).next().unwrap().text();
    //    println!("Node {:?}", node);
    println!();
    // let peripherals = Peripherals::take().unwrap();
    // let pins = peripherals.pins;
    //}
    //loop {
        info!("About to fetch content from {}", url);

        let mut client = EspHttpClient::new_default()?;

        let response = client.get(&url)?.submit()?;

        let body: Result<Vec<u8>, _> = Bytes::<_, 8>::new(response.reader()).collect();

        let body = body?;
        let str = String::from_utf8(body)?;
        //let str = String::from_utf8_lossy(&body)?;

        //let document = Html::parse_document(&body);
        //let document = Dom::parse(&String::from_utf8_lossy(&body))?;

        let soup = Soup::new(&str);
    //println!("Soup {:?}", soup);
    //let times = soup.tag("table").find_all().nth(n);

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

    info!(
        "About to initialize the ILI9341 SPI LED driver",
    );

    let config = <spi::config::Config as Default>::default()
        .baudrate((26_000_000).into());

    info!(
        "Info 1",
    );
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
        Orientation::Portrait,
        ili9341::DisplaySize240x320,
    ).map_err(|_| anyhow::anyhow!("Display"))?;

    //end of display


    let mut merged_string = String::new();
    let mut merge_counter = 0;
    for (_, link) in soup.tag("tr").find_all().enumerate() {
        let href = link.tag("h3").find_all().enumerate();

        for (_, node) in href {
            let text = &node.text();
            println!("txt: {:?}", &text);

            if merge_counter != 3 {
                merge_counter+=1;
                merged_string+=text;
            }
            else {
                merged_string.replace("\n", " ");
                let re = Regex::new(r"\s+").unwrap();
                let t = re.replace_all(&merged_string, " ").to_string();
                println!("Merged string :{:?}", t);
                led_draw(&mut display, &t);
                info!("About to sleep for 3 secs");
                std::thread::sleep(Duration::from_secs(3));
                merged_string.clear();
                merged_string+=text;
                merge_counter = 1;
            }

            info!("Dalsie kolo");

        }
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
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(_ip_settings))),
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
fn led_draw<D>(display: &mut D, text: &String) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    //let rect = Rectangle::new(display.bounding_box().top_left, display.bounding_box().size);

    display.clear(Rgb565::BLACK.into())?;
    //display.fill_solid(&rect, Rgb565::GREEN.into());

    Rectangle::new(display.bounding_box().top_left, display.bounding_box().size)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE.into())
                .stroke_color(Rgb565::YELLOW.into())
                .stroke_width(1)
                .build(),
        )
        .draw(display)?;

    Text::new(
        text,
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&embedded_graphics::mono_font::iso_8859_2::FONT_10X20, Rgb565::WHITE.into()),
    )
    .draw(display)?;

    info!("LED rendering done");

    Ok(())
}