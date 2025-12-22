#!/usr/bin/env bash
set -x
set -eo pipefail

# check redis server info
valkey-cli INFO server
valkey-cli PING