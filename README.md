# Experimental demo using ESP32-wrover kit and Rust

The main goal of the demo is to display line directions and departure times of public transport vehicles in Brno.

### What the project does:
- Establish WiFi connection
- Configure ILI9341 display, HTTP Client and SNTP Client
- parse data from website (web scraping)
- init RTC and display current time with parsed data


# Hardware setup

### HW: Wrover-kit
https://docs.espressif.com/projects/esp-idf/en/latest/esp32/hw-reference/esp32/get-started-wrover-kit.html

![demo](https://user-images.githubusercontent.com/43887390/165057872-e0d3bb64-a807-42d6-a7a4-b280e6ea9cac.jpg)

### Used pins
| ILI9341 |  Wrover-kit         |
----------|---------------------|
| RST     | GPIO18              |
| CLK     | GPIO19              |
| D_C     | GPIO21              |
| CS      | GPIO22              |
| MOSI    | GPIO23              |
| MISO    | GPIO25              |

### HW: ESP32 with ILI9341 display

<img width="723" alt="wokwi" src="https://user-images.githubusercontent.com/43887390/170480059-b26ba8f8-fe7b-49c7-80c2-91c5f25cc8f3.png">


## Wokwi
### Used pins
| ILI9341 |  ESP32-DevKitS-V1.1 | Cable color |
----------|---------------------|-------------|
| GND     | GND                 | black       |
| 3.3V    | 3.3V                | red         |
| RST     | GPIO4               | green       |
| CLK     | GPIO18              | orange      |
| D_C     | GPIO2               | purple      |
| CS      | GPIO15              | blue        |
| MOSI    | GPIO23              | yellow      |
| MISO    | GPIO25              | cyan        |

### diagram.json

```json
{
  "version": 1,
  "author": "Juraj Sadel",
  "editor": "wokwi",
  "parts": [
    {
      "type": "wokwi-esp32-devkit-v1",
      "id": "esp",
      "top": 0,
      "left": 0.67,
      "attrs": { "builder": "esp32-rust" }
    },
    { "type": "wokwi-ili9341", "id": "lcd1", "top": -354.05, "left": 182.84, "attrs": {} }
  ],
  "connections": [
    [ "esp:TX0", "$serialMonitor:RX", "", [] ],
    [ "esp:RX0", "$serialMonitor:TX", "", [] ],
    [ "esp:D4", "lcd1:RST", "green", [ "h0" ] ],
    [ "esp:3V3", "lcd1:VCC", "red", [ "v-1.02", "h132.77" ] ],
    [ "esp:GND.1", "lcd1:GND", "black", [ "h141.9", "v-1.79" ] ],
    [ "esp:D15", "lcd1:CS", "blue", [ "h0" ] ],
    [ "esp:D2", "lcd1:D/C", "purple", [ "h0" ] ],
    [ "esp:D18", "lcd1:SCK", "orange", [ "h0" ] ],
    [ "esp:D25", "lcd1:MISO", "cyan", [ "h-26.13", "v-106.89", "h328.67" ] ],
    [ "lcd1:MOSI", "esp:D23", "yellow", [ "v0" ] ],
    [ "lcd1:LED", "esp:3V3", "pink", [ "v226.59", "h-190.8" ] ]
  ],
  "serialMonitor": { "display": "terminal" }
}
```

### Wokwi project link
https://wokwi.com/projects/332616143815574099


# Devcontainers
 The repository supports:
<!-- GIPOR LINK NEEDS TO BE UPDATED WHEN MERGED -->
-  [Gitpod](https://gitpod.io/): [![Open ESP32 in Gitpod](https://gitpod.io/button/open-in-gitpod.svg)](https://gitpod.io/github.com/JurajSadel/wrover-experimental)
-  [Vs Code Devcontainers](https://code.visualstudio.com/docs/remote/containers#_quick-start-open-an-existing-folder-in-a-container)
-  [GitHub Codespaces](https://docs.github.com/en/codespaces/developing-in-codespaces/creating-a-codespace)
## Build
- Terminal approach:

    ```
    ./build.sh  [debug | release]
    ```
    > If no argument is passed, `release` will be used as default

-  UI approach:

    The default build task is already set to build the project, and it can be used
    in VsCode and Gitpod:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Build Task` command.
    - `Terminal`-> `Run Build Task` in the menu.
    - With `Ctrl-Shift-B` or `Cmd-Shift-B`.
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build`.
    - From UI: Press `Build` on the left side of the Status Bar.

## Flash

> **Note**
> When using GitHub Codespaces, we need to make the ports
> public, [see instructions](https://docs.github.com/en/codespaces/developing-in-codespaces/forwarding-ports-in-your-codespace#sharing-a-port).
- Terminal approach:
  - Using custom `runner` in `.cargo/config.toml`:
    ```
    cargo +esp run [--release]
    ```
  - Using `flash.sh` script:

    ```
    ./flash.sh [debug | release]
    ```
    > If no argument is passed, `release` will be used as default
- UI approach:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build & Flash`.
    - From UI: Press `Build & Flash` on the left side of the Status Bar.


## Wokwi Simulation

- Terminal approach:

    ```
    ./run-wokwi.sh [debug | release]
    ```
    > If no argument is passed, `release` will be used as default
- UI approach:

    The default test task is already set to build the project, and it can be used
    in VsCode and Gitpod:
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Test Task` command
    - With `Ctrl-Shift-,` or `Cmd-Shift-,`
        > Note: This Shortcut is not available in Gitpod by default.
    - From the [Command Palette](https://code.visualstudio.com/docs/getstarted/userinterface#_command-palette) (`Ctrl-Shift-P` or `Cmd-Shift-P`) run the `Tasks: Run Task` command and
    select `Build & Run Wokwi`.
    - From UI: Press `Build & Run Wokwi` on the left side of the Status Bar.
### Debuging with Wokwi

Wokwi offers debugging with GDB.

- Terminal approach:
    ```
    $HOME/.espressif/tools/xtensa-esp32-elf/esp-2021r2-patch3-8.4.0/xtensa-esp32-elf/bin/xtensa-esp32-elf-gdb target/xtensa-esp32-espidf/release/brno-public-transport -ex "target remote localhost:9333"
    ```
    > [Wokwi Blog: List of common GDB commands for debugging.](https://blog.wokwi.com/gdb-avr-arduino-cheatsheet/?utm_source=urish&utm_medium=blog)
- UI approach:
    Debug using with VsCode or Gitpod is also possible:
    1. Run the Wokwi Simulation in `debug` profile
        > Note that the simulation will pause if the browser tab is on the background
    2. Go to `Run and Debug` section of the IDE (`Ctrl-Shift-D or Cmd-Shift-D`)
    3. Start Debugging (`F5`)
    4. Choose the proper user:
        - `esp` when using VsCode or GitHub Codespaces
        - `gitpod` when using Gitpod
