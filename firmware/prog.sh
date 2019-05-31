#!/bin/env bash
arm-none-eabi-objcopy  -O binary -S target/thumbv6m-none-eabi/release/ffp ffp.bin
dfu-util -a 0 --dfuse-address 0x08000000 -D ffp.bin
