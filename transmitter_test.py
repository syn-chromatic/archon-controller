import socket

from abc import ABC, abstractmethod
from io import BufferedReader


class ControllerData(ABC):
    def __init__(self):
        pass

    @abstractmethod
    def get_bytes(self) -> bytearray:
        pass


class ASCIIData(ControllerData):
    def __init__(self, ascii: str):
        self.type: int = 0x02
        self.ascii: str = ascii

    def get_bytes(self) -> bytearray:
        type_bytes = self.type.to_bytes(2, "big")
        ascii_bytes = self.ascii.encode("ascii")
        return bytearray(type_bytes + ascii_bytes)


class ArchonTransmitter:
    def __init__(self, length: int, ip: str, port: int):
        self.length: int = length
        self.ip: str = ip
        self.port: int = port
        self.sender: socket.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.connected = False

    def get_file_size(self, file: BufferedReader) -> int:
        file.seek(0, 2)
        size = file.tell()
        file.seek(0)
        return size

    def print_incomplete_chunk(self, chunk: int, sent: int):
        print(f"Incomplete chunk sent! => Chunk: {chunk} | Sent: {sent}")

    def connect(self):
        if not self.connected:
            self.sender.connect((self.ip, self.port))
            self.connected = True

    def send(self, data: ControllerData):
        self.connect()
        data_bytes = data.get_bytes()

        print("-----------------------------------")
        print("Archon Controller")
        print("-----------------------------------")
        sent = self.sender.send(data_bytes)

        if sent != len(data_bytes):
            self.print_incomplete_chunk(len(data_bytes), sent)

        # self.sender.close()


length = 32
receiver_ip = "192.168.2.79"
receiver_port = 9688

archon = ArchonTransmitter(length, receiver_ip, receiver_port)

while True:
    data = input("Input: ")
    data = data[:1]
    data = ASCIIData(data)
    archon.send(data)
