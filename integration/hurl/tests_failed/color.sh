#!/bin/bash
set -Eeuo pipefail
hurl tests_failed/color.hurl --color
