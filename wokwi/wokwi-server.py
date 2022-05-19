#!/usr/bin/env python

import asyncio
import base64
import json
import sys
import os
import subprocess
import websockets
import webbrowser
import time
from gdbserver import GDBServer

PORT = 9012
GDB_PORT = 9333

def base64_file(path: str):
    with open(path, 'rb') as file:
        return base64.b64encode(file.read()).decode('ascii')

gdb_server = GDBServer()

async def handle_client(websocket, path):
    msg = await websocket.recv()
    print("Client connected! {}".format(msg))
    print("Elf file: {}/{}.elf".format(os.getenv('CURRENT_PROJECT'), os.getenv('CURRENT_PROJECT')))
    # Send the simulation payload
    await websocket.send(json.dumps({
        "type": "start",
        "elf": base64_file('/home/esp/workspace/target/xtensa-esp32-espidf/debug/brno-public-transport'.format(os.getenv('CURRENT_PROJECT'), os.getenv('CURRENT_PROJECT'))),
        "espBin": [
            #[0x1000, base64_file('{}/build/bootloader/bootloader.bin'.format(os.getenv('CURRENT_PROJECT')))],
            #[0x8000, base64_file('{}/build/partition_table/partition-table.bin'.format(os.getenv('CURRENT_PROJECT')))],
            #[0x10000, base64_file('{}/build/{}.bin'.format(os.getenv('CURRENT_PROJECT'), os.getenv('CURRENT_PROJECT')))],
            [0x00000, base64_file('app_patched9.bin')],
        ]
    }))

    gdb_server.on_gdb_message = lambda msg: websocket.send(
        json.dumps({"type": "gdb", "message": msg}))
    gdb_server.on_gdb_break = lambda: websocket.send(
        json.dumps({"type": "gdbBreak"}))

    while True:
        msg = await websocket.recv()
        msgjson = json.loads(msg)
        if msgjson["type"] == "uartData":
            sys.stdout.buffer.write(bytearray(msgjson["bytes"]))
            sys.stdout.flush()
        elif msgjson["type"] == "gdbResponse":
            await gdb_server.send_response(msgjson["response"])
        else:
            print("> {}".format(msg))

start_server = websockets.serve(handle_client, "127.0.0.1", PORT)
asyncio.get_event_loop().run_until_complete(start_server)


board = 329992166987268691
if(os.getenv('USER') == "gitpod"):
    gp_url = subprocess.getoutput("gp url {}".format(PORT))
    gp_url = gp_url[8:]
    url = "https://wokwi.com/_alpha/wembed/{}?partner=espressif&port={}&data=demo&_host={}".format(board,PORT,gp_url)
else:
    url = "https://wokwi.com/_alpha/wembed/{}?partner=espressif&port={}&data=demo".format(board,PORT)

print("Please, open the following URL: {}".format(url))
if(os.getenv('USER') == "gitpod"):
    time.sleep(2)
    open_preview = subprocess.getoutput("gp preview \"{}\"".format(url))
else:
    webbrowser.open(url)
asyncio.get_event_loop().run_until_complete(gdb_server.start(GDB_PORT))
asyncio.get_event_loop().run_forever()