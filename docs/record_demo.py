# /// script
# requires-python = ">=3.13"
# dependencies = ["pyserial"]
# ///

import sys
import time
import threading
import subprocess
import serial

PORT = sys.argv[1] if len(sys.argv) > 1 else "/dev/ttyUSB1"
TAPE = sys.argv[2] if len(sys.argv) > 2 else "demo.tape"
BAUD = 115200

LOREM = [
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
    "Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
    "Ut enim ad minim veniam, quis nostrud exercitation ullamco.",
    "Duis aute irure dolor in reprehenderit in voluptate velit esse.",
    "Excepteur sint occaecat cupidatat non proident, sunt in culpa.",
    "Curabitur pretium tincidunt lacus. Nulla gravida orci a odio.",
    "Nullam varius, turpis molestie dictum semper, turpis justo.",
    "Phasellus at purus et libero lacinia dictum. Fusce aliquet.",
]

VHS_TO_CONNECT_SECS = 6.0

def write(ser: serial.Serial, line: str, delay_after: float = 0.3):
    ser.write((line + "\r\n").encode())
    ser.flush()
    time.sleep(delay_after)

def feeder(start_event: threading.Event, stop_event: threading.Event):
    print(f"[feeder] Opening {PORT} at {BAUD} baud...")
    with serial.Serial(PORT, BAUD, timeout=1) as ser:
        start_event.wait()
        time.sleep(VHS_TO_CONNECT_SECS)

        write(ser, "\r\nHello World!\r\n", delay_after=1.0)

        i = 0
        while not stop_event.is_set():
            write(ser, LOREM[i % len(LOREM)] + "\r\n", delay_after=0.1)
            i += 1

    print("[feeder] Done.")

def recorder(start_event: threading.Event, stop_event: threading.Event):
    print(f"[recorder] Starting vhs {TAPE}...")
    start_event.set()
    result = subprocess.run(["vhs", TAPE])
    stop_event.set()
    if result.returncode != 0:
        print(f"[recorder] vhs exited with code {result.returncode}")
    else:
        print("[recorder] Done.")

def main():
    start_event = threading.Event()
    stop_event = threading.Event()

    t_feeder = threading.Thread(target=feeder, args=(start_event, stop_event), daemon=True)
    t_recorder = threading.Thread(target=recorder, args=(start_event, stop_event))

    t_feeder.start()
    t_recorder.start()

    t_recorder.join()
    t_feeder.join(timeout=5)

main()
