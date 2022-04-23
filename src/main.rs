#![feature(iter_advance_by)]
use embedded_graphics::mono_font::iso_8859_2;
use esp_idf_sys::{
    self as _,};
use esp_idf_sys::time_t;

use embedded_svc::http::client::*;
use embedded_svc::io::Bytes;
use embedded_svc::wifi::*;

use esp_idf_svc::http::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;
use esp_idf_svc::sntp;
use esp_idf_svc::sntp::SyncStatus;

use esp_idf_hal::spi;
use esp_idf_hal::delay;
use esp_idf_hal::prelude::Peripherals;

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

use std::sync::Arc;
use std::time::Duration;
use std::ptr;

use time::OffsetDateTime;
use time::format_description;

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

    /*
    1 initialization(wifi, display, peripherals,....)
    2 loop
        3 fetch data from url
        4 reload vector (5 lines)
        5 display new results
        6 sleep for 45-60 secs
    */
    //unsafe extern "C" fn time(_timer: *mut time_t) -> time_t

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

    info!("Waiting done");

    unsafe {
        let timer: *mut time_t = ptr::null_mut();
        //esp_idf_sys::time(timer);
        println!("Timer0: {:?}", esp_idf_sys::time(timer));
        //let timer: 
        //let mut t;
        // for _ in 0..1000 {
        //     t = esp_idf_sys::time(timer);
        //     println!("Timer :{:?}", t);
        // }
    }

    

    unsafe {
        let timer: *mut time_t = ptr::null_mut();
        let time = esp_idf_sys::time(timer);
        println!("Timer1: {:?}", time);
        //let timer: 
        //let mut t;
        //for _ in 0..1000 {
        //     t = esp_idf_sys::time(timer);
        //     println!("Timer :{:?}", t);
        // }
        use time::macros::offset;
        let date = OffsetDateTime::from_unix_timestamp(time as i64)?.to_offset(offset!(+2)).time();
        println!("date1 {:?}", &date.to_string());

        std::thread::sleep(Duration::from_secs(5));


        //let a = OffsetDateTime::from("14:31:58.0");
        //22.4.2022 12:44:00
        //"2020-01-02 03:04:05 +06:07:08"
        // let format = format_description::parse(
        //     "[day].[month].[year] [hour]:[minute]:[second] [offset_hour \
        //     sign:mandatory]:[offset_minute]:[offset_second]",
        // )?;
                                                            //23.04.2022 20:16:00 +02:00:00
        let qq = OffsetDateTime::parse("22.04.2022 12:44:00 +02:00:00", &format)?;
        let qq = qq.time();
        println!("Date1: {:?}", &qq.to_string());

        let timer: *mut time_t = ptr::null_mut();
        let time = esp_idf_sys::time(timer);
        let qq = OffsetDateTime::from_unix_timestamp(time as i64)?.to_offset(offset!(+2)).time();
        println!("date2 {:?}", &qq.to_string());

        if date > qq {
            println!("Hej");
        }
        else {
            println!("Niet");
        }

    }

    



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
    let config = <spi::config::Config as Default>::default()
    .baudrate((26_000_000).into());

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
    ).map_err(|_| anyhow::anyhow!("Display"))?;
    
    // use time::UtcOffset;
    // let local_offset = current_local_offset();
    // println!("{:?}, time", local_offset);

    let url = String::from("https://idos.idnes.cz/en/brno/odjezdy/vysledky/?f=Technologick%C3%BD%20park&fc=302003");

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

    

    // display

    info!(
        "About to initialize the ILI9341 SPI LED driver",
    );

    //let mut scroller = display.configure_vertical_scroll(20, 5).map_err(|_| anyhow::anyhow!("Display"))?;
    // display.scroll_vertically(scroller, 5);

    //end of display


    let mut merged_string = String::from(" \n".to_string());
    let mut merge_counter = 0;
    let mut display_counter = 0;
    for link in soup.tag("tr").find_all() {
        //println!("Link: {:?}\n", link.display());
        //let tmp = link.class("dep-row dep-row-first").find_all();
        //let tmp = link.attr_name("data-datetime").find_all();
        //let tmp1 = link.attr_value("data-datetime").find_all();
        let qwe = link.attr_name("data-datetime").find_all();
        link.display();
        println!("\n\n");
        for q in qwe {
            q.display();
        }

        let l = link.attrs();

        //let atr = link.get("data-datetime");
        //println!("Attr = {:?}", &atr.unwrap());
        // for a in tmp {
        //     a.display();
        // }
        
        
        /*let tmp = link.attrs();*/
        for (s1, s2) in l {
            println!("S1 {:?}/tS2{:?}", &s1, &s2);
        }
        

        //println!("tmp: {:?}", tmp.display());
        //let href = link.tag("h3").find_all().enumerate();

        // for node in tmp {
        //     let text = &node.text();
        //     //println!("Node: {:?}", node.display());
        //     let re = Regex::new(r"\s+").unwrap();
        //     let t = re.replace_all(&text, " ").to_string();

        //     if merge_counter != 3 {
        //         merged_string+=&t;
        //         merge_counter+=1;

        //     }
        //     else {
        //         //println!("Merged string :{:?}", merged_string);
        //         //led_draw(&mut display, &merged_string);
        //         info!("About to sleep for 1 sec");
        //         //std::thread::sleep(Duration::from_secs(1));
        //         //merged_string.clear();
        //         merged_string+=" \n";
        //         merged_string+=text;
        //         merged_string+="\n";
        //         merge_counter = 1;
        //         display_counter+=1;

        //         if display_counter == 5 {
        //             led_draw(&mut display, &merged_string);
        //             break;
        //         }
        //     }
        // }
        //display.scroll_vertically(&mut scroller, 20);
        //info!("Dalsie kolo");



        /************** */

        // for (_, node) in href {
        //     let text = &node.text();
        //     println!("txt: {:?}", &text);

        //     if merge_counter != 3 {
        //         merge_counter+=1;
        //         merged_string+=text;
        //     }
        //     else {
        //         //merged_string.replace("\n", " ");
        //         let re = Regex::new(r"\s+").unwrap();
        //         let t = re.replace_all(&merged_string, " ").to_string();
        //         println!("Merged string :{:?}", t);
        //         led_draw(&mut display, &t);
        //         info!("About to sleep for 3 secs");
        //         std::thread::sleep(Duration::from_secs(3));
        //         merged_string.clear();
        //         merged_string+=text;
        //         merge_counter = 1;
        //     }
        //     display.scroll_vertically(&mut scroller, 5);
        //     info!("Dalsie kolo");

        // }



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

    //display.clear(Rgb565::BLACK.into())?;
    //display.fill_solid(&rect, Rgb565::GREEN.into());

    Rectangle::new(Point::zero(), Size::new(300, 320))
        .into_styled(
            TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Justified)
        .paragraph_spacing(3)
        .build(),
        );
        //.draw(display)?;

    Text::with_alignment(
        &text,
        Point::new(3, 0),
        MonoTextStyle::new(&embedded_graphics::mono_font::iso_8859_2::FONT_10X20, Rgb565::WHITE.into()),
        Alignment::Left,
    ).draw(display)?;

    // let character_style = MonoTextStyle::new(&embedded_graphics::mono_font::iso_8859_2::FONT_10X20, Rgb565::WHITE.into());
    // //let textbox_style = TextBoxStyleBuilder::new()
    //  //   .height_mode(HeightMode::FitToText)
    //  //   .alignment(HorizontalAlignment::Justified)
    //  //   .paragraph_spacing(6)
    //  //   .build();

    // Rectangle::new(Point::zero(), Size::new(display.bounding_box().size.width, display.bounding_box().size.height))
    // .into_styled(
    //     TextBoxStyleBuilder::new()
    //     .height_mode(HeightMode::FitToText)
    //     .alignment(HorizontalAlignment::Justified)
    //     .paragraph_spacing(6)
    //     .build(),
    // )
    // .draw(display)?;

    // let bounds = Rectangle::new(Point::zero(), Size::new(display.bounding_box().size.width, display.bounding_box().size.height));
    // let text_box = TextBox::with_textbox_style(text, bounds, character_style, textbox_style);

    // //let mut display = SimulatorDisplay::new(text_box.bounding_box().size);
    // text_box.draw(&mut display).unwrap();

    // Text::new(
    //     text,
    //     Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
    //     MonoTextStyle::new(&embedded_graphics::mono_font::iso_8859_2::FONT_10X20, Rgb565::WHITE.into()),
    // )
    // .draw(display)?;

    info!("LED rendering done");

    Ok(())
}