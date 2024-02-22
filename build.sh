#!/bin/bash

set -ex

svd patch svd/sg2002.yaml


svd2rust --target riscv -g --strict --pascal_enum_values --max_cluster_size -o pac -i ./svd/SG2002.svd.patched


cargo fmt
