# Experimental demo using ESP32-wrover kit and Rust

The main goal of the demo is to display line directions and departure times of public transport vehicles in Brno.
![demo](https://user-images.githubusercontent.com/43887390/165057872-e0d3bb64-a807-42d6-a7a4-b280e6ea9cac.jpg)

Most important steps:
- Establish WiFi connection
- Configure ILI9341 display, HTTP Client and SNTP Client
- parse data from website (web scraping)
- init RTC and display current time with parsed data 
