#!/usr/bin/env bash
set -x
set -eo pipefail

# optional, setup user
sudo useradd -r -s /usr/sbin/nologin -M telegrab
