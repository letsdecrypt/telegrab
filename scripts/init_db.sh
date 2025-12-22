#!/usr/bin/env bash
set -x
set -eo pipefail

sudo -u postgres psql -c "CREATE USER telegrab WITH PASSWORD 'password';"
sudo -u postgres psql -c "CREATE DATABASE telegrab OWNER telegrab;"

sqlx database create
sqlx migrate run