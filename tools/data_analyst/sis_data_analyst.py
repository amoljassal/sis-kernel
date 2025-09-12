#!/usr/bin/env python3
"""
Simple Data Analyst pipeline orchestrator (userspace skeleton).
Builds a pipeline via control-plane V0 frames over /tmp/sis-datactl.sock.

Stages: acquire, clean, explore, model, explain.
"""
import socket
import struct

SOCK_PATH = '/tmp/sis-datactl.sock'

def _frame(cmd: int, payload: bytes, flags: int = 0) -> bytes:
    return struct.pack('<BBBBI', 0x43, 0, cmd, flags, len(payload)) + payload

def _send(cmd: int, payload: bytes):
    data = _frame(cmd, payload)
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(SOCK_PATH)
        s.sendall(data)

class Graph:
    def __init__(self):
        _send(0x01, b'')  # CreateGraph
        self.channels = 0
        self.ops = 0

    def add_channel(self, capacity: int) -> int:
        if capacity < 1 or capacity > 65535:
            raise ValueError('capacity must be 1..65535')
        _send(0x02, struct.pack('<H', capacity))
        idx = self.channels
        self.channels += 1
        return idx

    def add_operator(self, op_id: int, stage: str, in_ch: int | None, out_ch: int | None, priority: int = 10) -> int:
        stage_map = { 'acquire':0,'clean':1,'explore':2,'model':3,'explain':4 }
        st = stage_map.get(stage, 0)
        inv = 0xFFFF if in_ch is None else int(in_ch)
        outv = 0xFFFF if out_ch is None else int(out_ch)
        _send(0x03, struct.pack('<IHHBB', int(op_id), inv, outv, int(priority), st))
        idx = self.ops
        self.ops += 1
        return idx

    def start(self, steps: int):
        _send(0x04, struct.pack('<I', int(steps)))

    # Convenience wrappers for the five stages
    def acquire(self, out_ch: int | None, priority: int = 10) -> int:
        return self.add_operator(self.ops + 1, 'acquire', None, out_ch, priority)
    def clean(self, in_ch: int, out_ch: int | None, priority: int = 10) -> int:
        return self.add_operator(self.ops + 1, 'clean', in_ch, out_ch, priority)
    def explore(self, in_ch: int, out_ch: int | None, priority: int = 10) -> int:
        return self.add_operator(self.ops + 1, 'explore', in_ch, out_ch, priority)
    def model(self, in_ch: int, out_ch: int | None, priority: int = 10) -> int:
        return self.add_operator(self.ops + 1, 'model', in_ch, out_ch, priority)
    def explain(self, in_ch: int, out_ch: int | None, priority: int = 10) -> int:
        return self.add_operator(self.ops + 1, 'explain', in_ch, out_ch, priority)

if __name__ == '__main__':
    # Example: Acquire -> Clean -> Model
    g = Graph()
    ch0 = g.add_channel(64)
    op0 = g.acquire(out_ch=ch0)
    op1 = g.clean(in_ch=ch0, out_ch=None)
    g.start(128)
    print('Pipeline started (Acquire->Clean)')

