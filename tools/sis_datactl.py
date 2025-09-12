#!/usr/bin/env python3
import argparse
import socket
import struct

SOCK_PATH = '/tmp/sis-datactl.sock'

def frame(cmd: int, payload: bytes, flags: int = 0) -> bytes:
    # magic 'C'(0x43), ver=0, cmd, flags, len (LE u32), payload
    hdr = struct.pack('<BBBBI', 0x43, 0, cmd, flags, len(payload))
    return hdr + payload

def send_frame(cmd: int, payload: bytes):
    data = frame(cmd, payload)
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(SOCK_PATH)
        s.sendall(data)

def cmd_create_graph(args):
    send_frame(0x01, b'')
    print('CreateGraph sent')

def cmd_add_channel(args):
    cap = int(args.capacity)
    if cap < 1 or cap > 65535:
        raise SystemExit('capacity must be 1..65535')
    payload = struct.pack('<H', cap)
    send_frame(0x02, payload)
    print(f'AddChannel(capacity={cap}) sent')

def cmd_add_operator(args):
    op_id = int(args.op_id)
    in_ch = int(args.in_ch) if args.in_ch is not None else 0xFFFF
    out_ch = int(args.out_ch) if args.out_ch is not None else 0xFFFF
    prio = int(args.priority)
    stage_map = {
        'acquire': 0,
        'clean': 1,
        'explore': 2,
        'model': 3,
        'explain': 4,
        None: 0,
    }
    stage = stage_map.get(args.stage, 0)
    payload = struct.pack('<IHHBB', op_id, in_ch, out_ch, prio, stage)
    send_frame(0x03, payload)
    print(f'AddOperator(op_id={op_id}, in={in_ch}, out={out_ch}, prio={prio}, stage={stage}) sent')

def cmd_start(args):
    steps = int(args.steps)
    payload = struct.pack('<I', steps)
    send_frame(0x04, payload)
    print(f'StartGraph(steps={steps}) sent')

def main():
    ap = argparse.ArgumentParser(description='SIS control-plane client (V0 framing)')
    sub = ap.add_subparsers(dest='cmd', required=True)

    sub.add_parser('create').set_defaults(fn=cmd_create_graph)

    ap_ch = sub.add_parser('add-channel')
    ap_ch.add_argument('capacity')
    ap_ch.set_defaults(fn=cmd_add_channel)

    ap_op = sub.add_parser('add-operator')
    ap_op.add_argument('op_id')
    ap_op.add_argument('--in-ch', type=int)
    ap_op.add_argument('--out-ch', type=int)
    ap_op.add_argument('--priority', type=int, default=10)
    ap_op.add_argument('--stage', choices=['acquire','clean','explore','model','explain'])
    ap_op.set_defaults(fn=cmd_add_operator)

    ap_run = sub.add_parser('start')
    ap_run.add_argument('steps')
    ap_run.set_defaults(fn=cmd_start)

    args = ap.parse_args()
    args.fn(args)

if __name__ == '__main__':
    main()
