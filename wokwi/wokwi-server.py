#!/usr/bin/env python

import asyncio
import base64
import json
import sys
import os
import websockets
import webbrowser


PORT = 9012


def base64_file(path: str):
    with open(path, 'rb') as file:
        return base64.b64encode(file.read()).decode('ascii')


async def hello(websocket, path):
    msg = await websocket.recv()
    print("Client connected! {}".format(msg))

    # Send the simulation payload
    await websocket.send(json.dumps({
        "type": "start",
        "elf": base64_file('{}/wokwi/dummy.elf'.format(os.getcwd())),
        "espBin": [
            [0x0, base64_file('merged_32.bin')],
            #[0x1000, base64_file('{}/config-files/bootloader-esp32.bin'.format(os.getcwd()))],
            #[0x8000, base64_file('{}/config-files/partition-table-esp32.bin'.format(os.getcwd()))],
            #[0x10000, base64_file('{}/wr.bin'.format(os.getenv('CURRENT_PROJECT')))],
        ]
    }))

    while True:
        msg = await websocket.recv()
        msgjson = json.loads(msg)
        if msgjson["type"] == "uartData":
            sys.stdout.buffer.write(bytearray(msgjson["bytes"]))
            sys.stdout.flush()
        else:
            print("> {}".format(msg))

#https://wokwi.com/projects/330820329911878226
start_server = websockets.serve(hello, "127.0.0.1", PORT)
asyncio.get_event_loop().run_until_complete(start_server)
board = 325149339656651346
if os.getenv('ESP_BOARD') == "esp32c3":
    board = 325149339656651346
elif os.getenv('ESP_BOARD') == "esp32c3-rust":
    board = 328638850887844436
elif os.getenv('ESP_BOARD') == "esp32":
    board = 330820329911878226
# else :
#     return 1

url = "https://wokwi.com/_alpha/wembed/{}?partner=espressif&port={}&data=demo".format(board,PORT)
print("Web socket listening on port {}".format(PORT))
print("")
print("Please, open the following URL: {}".format(url))
webbrowser.open(url)
asyncio.get_event_loop().run_forever()