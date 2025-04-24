#!/bin/sh

cargo objcopy --release -- -O binary usb_kb_demo.bin && uf2conv usb_kb_demo.bin --base 0x4000 --output usb_kb_demo.uf2
