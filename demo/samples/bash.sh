#!/usr/bin/env bash

[[ "$TRACE" ]] && set -o xtrace
set -o errexit
set -o nounset
set -o pipefail
set -o noclobber

if [[ "${EUID}" -ne 0 ]]; then
  echo "ERROR: this script must run as root" >>/dev/stderr
  exit 1
fi

if [[ -e /sys/class/gpio/gpio17 ]]; then
  echo "Unexporting pin 17 first"
  set +o noclobber
  echo 17 > /sys/class/gpio/unexport
  set -o noclobber
fi

echo "Setting up pin 17"
set +o noclobber
echo 17 >/sys/class/gpio/export
echo "in" >/sys/class/gpio/gpio17/direction
echo "both" >/sys/class/gpio/gpio17/edge
set -o noclobber
echo "Pin 17 setup complete"

while inotifywait -e modify /sys/class/gpio/gpio17/value
do
  echo "Doorbell Ring Started"
  mosquitto_pub -h mqtt1 -t stat/front-door/RING -m ON
  sleep 0.5
  mosquitto_pub -h mqtt1 -t stat/front-door/RING -m OFF
  echo "Doorbell Ring Finished"

  # Fake debouncing, only allow triggering every ~10 seconds
  echo "Fake debouncing"
  sleep 10
done
