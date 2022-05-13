import asyncio
import socket
from typing import Awaitable, Callable, Optional


def gdbChecksum(text: str):
    return format(sum(map(ord, text)) & 0xff, '02x')


class GDBServer:
    on_gdb_break: Optional[Callable[[], Awaitable[None]]]
    on_gdb_message: Optional[Callable[[str], Awaitable[None]]]

    def __init__(self):
        self._client = None
        self.on_gdb_break = None
        self.on_gdb_message = None

    def log(self, message: str):
        print("[GDB] " + message)

    async def start(self, port: int, host='localhost'):
        self.server = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.server.bind((host, port))
        self.server.listen(1)
        self.server.setblocking(False)

        loop = asyncio.get_event_loop()

        while True:
            client, addr = await loop.sock_accept(self.server)
            self.log('Connected ({}:{})'.format(*addr))
            loop.create_task(self.handle_client(client))

    async def handle_client(self, client: socket):
        self._client = client
        loop = asyncio.get_event_loop()
        await loop.sock_sendall(client, '+'.encode('utf-8'))

        buf = ''
        while True:
            chunk = (await loop.sock_recv(client, 1024))
            if len(chunk) == 0:
                break  # Disconnected
            if (chunk[0] == 3):
                print("[GDB] BREAK")
                if self.on_gdb_break:
                    await self.on_gdb_break()
                chunk = chunk[1:]
            buf += chunk.decode('utf-8')
            dolla = buf.find('$')
            hash = buf.find('#', dolla)
            if (dolla < 0 or hash < 0 or hash + 2 > len(buf)):
                continue
            cmd = buf[dolla + 1: hash]
            cksum = buf[hash + 1: hash + 3]
            buf = buf[hash + 3:]
            if (gdbChecksum(cmd) != cksum):
                self.log('Warning: Checksum error in message: {}'.format(cmd))
                await loop.sock_sendall(client, '-'.encode('utf-8'))
            elif self.on_gdb_message:
                await loop.sock_sendall(client, '+'.encode('utf-8'))
                await self.on_gdb_message(cmd)
            else:
                self.log(
                    'Error: Wokwi Simulator is not connected; ignoring GDB message')
        self.log("Disconnected")
        if self._client == client:
            self.client = None

    async def send_response(self, msg: str):
        if self._client:
            await asyncio.get_event_loop().sock_sendall(self._client, msg.encode('utf-8'))
        else:
            self.log(
                'Error: Wokwi sent a GDB response, but GDB is disconnected')