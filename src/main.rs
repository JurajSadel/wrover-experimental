use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use embedded_svc::http::{self, client::*, status, Headers, Status};
use embedded_svc::io::Bytes;
use embedded_svc::wifi::*;
use embedded_svc::ping::Ping;

use esp_idf_svc::http::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;

use esp_idf_hal::gpio;
use esp_idf_hal::spi;
use esp_idf_hal::delay;
use esp_idf_hal::prelude::Peripherals;

use embedded_hal::digital::v2::OutputPin;

use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;

//use std::error::Error;
use std::sync::Arc;
use std::{thread, time::*};

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
    let mut wifi = wifi(
        netif_stack.clone(),
        sys_loop_stack.clone(),
        default_nvs.clone(),
    )?;

    let url = String::from("https://idos.idnes.cz/brno/odjezdy/vysledky/?f=Technologick%C3%BD%20park&fc=302003");

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
    //}
    //loop {
        info!("About to fetch content from {}", url);

        let mut client = EspHttpClient::new_default()?;

        let response = client.get(&url)?.submit()?;

        let body: Result<Vec<u8>, _> = Bytes::<_, 8>::new(response.reader()).collect();

        let body = body?;
        let str = String::from_utf8(body)?;

        //let document = Html::parse_document(&body);
        //let document = Dom::parse(&String::from_utf8_lossy(&body))?;

        let soup = Soup::new(&str);
    //println!("Soup {:?}", soup);
    //let times = soup.tag("table").find_all().nth(n);

    for (i, link) in soup.tag("tr").find_all().enumerate() {
        //println!("{:?}\n\n",link.display());
        let href = link.tag("h3").find_all().enumerate();
        
        for (u, node) in href {

            // println!("Href: {:?}", node.display());
            // match node.data {
            //     node::Text => {
            //         println!("NodeData");
            //     }
            //     node::NodeData => {
            //         println!("Text");
            //     }
            // }

            
            println!("Node: {:?}", &node.text());
            // if let Some(n) = node {
            //     println!("N, {:?}", n);
            // }
            
        }
    }

        //println!("Doc{:?}", &document);

    //     let html = include_str!("./index.html");
    // let dom = Dom::parse(html)?;
    // let iter = dom.children.get(0).unwrap().into_iter();

    // let hrefs = iter.filter_map(|item| match item {
    //     Node::Element(ref element) if element.name == "a" => element.attributes["href"].clone(),
    //     _ => None,
    // });


        // let dom = Dom::parse(&String::from_utf8_lossy(&body))?;
        // let iter = dom.children.get(0).unwrap().into_iter();
        
        // let class = vec!("departures-table__cell departures-table__cell--height-collapse");
        // let trams = iter.filter_map(|item| match item {
        //     Node::Element(ref element) if element.name == "td" => Some(element.classes[0].clone()),
        //     _ => None,
        // });

        // for (index, href) in trams.enumerate() {
        //     println!("{}: {}", index + 1, href)
        // }
    

        // for d in document.children {
        //     //let child = d.children.into_iter();
        //     println!("Child: {:?}", &d);
            
            
/*
Element(Element { id: None, name: "tr", variant: Normal, attributes: {"data-ttindex": Some("0"), "data-train": Some("5175"), "data-datetime": Some("31.3.2022 13:44:00"), "data-stationname": Some("Kom&#225;rov")} */

            //child.binary_search_by_key(b, f)
        //}

        //println!("Body: {:?}", String::from_utf8(body).unwrap());
        //let body = body?;
        // info!(
        //     "Body:\n{:?}",
        //     String::from_utf8(body).unwrap()
        // );
        //let response = client.get(&url)?.submit()?;
        //body = response.reader().into_iter().collect()/* .into_iter().collect()*/;


        
        info!("About to sleep");
        //thread::sleep(Duration::from_millis(1000));
        //println!(
        //    "Body :\n{:?}",
        //    String::from_utf8_lossy(&body).into_owned()
        //);

        //parse part

        //let soup = Soup::new(&html);
        /* 
        let soup = Soup::from_reader(response.reader().into()).unwrap();
        let result = soup
        .tag("section")
        .attr("id", "main")
        .find()
        .and_then(|section| {
            section
                .tag("span")
                .attr("class", "in-band")
                .find()
                .map(|span| span.text())
        });
        assert_eq!(result, Some("Crate soup".to_string()));
        let title = soup.tag("departures-table__cell").find().expect("Couldn't find tag departures-table__cell");
        */


        /*
        let css_selector = "dep-row dep-row-first";

        let document = kuchiki::parse_html().one(String::from_utf8_lossy(&body).into_owned());
        //let document = kuchiki::parse_html().one(html);
        for css_match in document.select(css_selector).unwrap() {
            // css_match is a NodeDataRef, but most of the interesting methods are
            // on NodeRef. Let's get the underlying NodeRef.
            let as_node = css_match.as_node();
    
            // In this example, as_node represents an HTML node like
            //
            //   <p class='foo'>Hello world!</p>"
            //
            // Which is distinct from just 'Hello world!'. To get rid of that <p>
            // tag, we're going to get each element's first child, which will be
            // a "text" node.
            //
            // There are other kinds of nodes, of course. The possibilities are all
            // listed in the `NodeData` enum in this crate.
            let text_node = as_node.first_child().unwrap();
    
            // Let's get the actual text in this text node. A text node wraps around
            // a RefCell<String>, so we need to call borrow() to get a &str out.
            let text = text_node.as_text().unwrap().borrow();
    
            // Prints:
            //
            //  "Hello, world!"
            //  "I love HTML"
            println!("{:?}", text);
        }
        */
//    }



    drop(wifi);
    info!("Wifi stopped");
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;

    kaluga_hello_world(
        pins.gpio6,
        pins.gpio13,
        pins.gpio16,
        peripherals.spi3,
        pins.gpio15,
        pins.gpio9,
        pins.gpio11,
    )?;

    println!("Hello, world!");


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

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

fn kaluga_hello_world(
    backlight: gpio::Gpio6<gpio::Unknown>,
    dc: gpio::Gpio13<gpio::Unknown>,
    rst: gpio::Gpio16<gpio::Unknown>,
    spi: spi::SPI3,
    sclk: gpio::Gpio15<gpio::Unknown>,
    sdo: gpio::Gpio9<gpio::Unknown>,
    cs: gpio::Gpio11<gpio::Unknown>,
) -> anyhow::Result<()> {
    info!(
        "About to initialize the ILI9341 SPI LED driver",
    );

    let config = <spi::config::Config as Default>::default()
        .baudrate((40_000_000).into());

    let mut backlight = backlight.into_output()?;
    backlight.set_high()?;

    let di = SPIInterfaceNoCS::new(
        spi::Master::<spi::SPI3, _, _, _, _>::new(
            spi,
            spi::Pins {
                sclk,
                sdo,
                sdi: Option::<gpio::Gpio21<gpio::Unknown>>::None,
                cs: Some(cs),
            },
            config,
        )?,
        dc.into_output()?,
    );

    let reset = rst.into_output()?;

    let mut display = ili9341::Ili9341::new(
        di,
        reset,
        &mut delay::Ets,
        Orientation::Landscape,
        ili9341::DisplaySize240x320,
    )?;

    led_draw(&mut display)

}

#[allow(dead_code)]
fn led_draw<D>(display: &mut D) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    display.clear(Rgb565::BLACK.into())?;

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
        "Hello Rust!",
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE.into()),
    )
    .draw(display)?;

    info!("LED rendering done");

    Ok(())
}