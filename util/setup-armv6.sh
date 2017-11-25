#!/bin/bash

set -e

if [ ! -d .cargo/armv6-toolchain ]; then
	svn export https://github.com/raspberrypi/tools/trunk/arm-bcm2708/gcc-linaro-arm-linux-gnueabihf-raspbian/ .cargo/armv6-toolchain
fi
