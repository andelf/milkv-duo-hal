#!/bin/bash

set -ex

FSPL_PATH=./pre-built/fsbl

# opensbi
MONITOR_RUNADDR=0x0000000080000000
# after uboot
BLCP_2ND_RUNADDR=0x0000000087e00000

python3 ./pre-built/fsbl/plat/sg200x/fiptool.py -v genfip \
        './fip.bin' \
        --MONITOR_RUNADDR="${MONITOR_RUNADDR}" \
        --BLCP_2ND_RUNADDR="${BLCP_2ND_RUNADDR}" \
        --CHIP_CONF="${FSPL_PATH}/build/milkv_duo_256m/chip_conf.bin" \
        --NOR_INFO='FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF' \
        --NAND_INFO='00000000'\
        --BL2="${FSPL_PATH}/build/milkv_duo_256m/bl2.bin" \
        --BLCP_IMG_RUNADDR=0x05200200 \
        --BLCP_PARAM_LOADADDR=0 \
        --BLCP="${FSPL_PATH}/test/empty.bin" \
        --DDR_PARAM="${FSPL_PATH}/test/sophgo/ddr_param.bin" \
        --MONITOR='./pre-built/fw_dynamic.bin' \
        --LOADER_2ND="./firmware.bin" \
        --compress='lzma'

#        --BLCP_2ND="firmware.bin" \
#--LOADER_2ND="./pre-built/u-boot-raw.bin" \
