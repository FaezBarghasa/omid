#!/bin/bash
# OMID Linux USB Gadget ConfigFS Setup Script
#
# Run this on a Raspberry Pi (Zero/4/5) or BeagleBone to expose it as a
# high-performance physical OMID (Object-MIDI) USB device.
#
# Requires: dwc2 / musb-hdrc drivers loaded.

set -e

CONFIGFS_HOME="/sys/kernel/config/usb_gadget"
GADGET_NAME="omid_gadget"
GADGET_DIR="${CONFIGFS_HOME}/${GADGET_NAME}"

if [ "$EUID" -ne 0 ]; then
  echo "Error: Please run as root."
  exit 1
fi

echo "Initializing OMID USB Gadget..."

# Create gadget directory
mkdir -p "${GADGET_DIR}"
cd "${GADGET_DIR}"

# Vendor & Product IDs (USB Compliance)
echo "0x1d6b" > idVendor  # Linux Foundation
echo "0x0104" > idProduct # Multifunction Composite Gadget
echo "0x0200" > bcdUSB
echo "0xef" > bDeviceClass
echo "0x02" > bDeviceSubClass
echo "0x01" > bDeviceProtocol

# English strings
mkdir -p strings/0x409
echo "1234567890" > strings/0x409/serialnumber
echo "Omid Consortium" > strings/0x409/manufacturer
echo "OMID Unified Audio & Control Endpoint" > strings/0x409/product

# Create configuration
mkdir -p configs/c.1
mkdir -p configs/c.1/strings/0x409
echo "OMID Transport Profile" > configs/c.1/strings/0x409/configuration
echo "500" > configs/c.1/MaxPower

# Expose WinUSB/OS Descriptors (for automatic Windows driver loading)
echo "1" > os_desc/use
echo "0xcd" > os_desc/b_vendor_code
echo "MSFT100" > os_desc/qw_sign

# Create function: MIDI 2.0 / Audio Class 2 (OMID) function
# We configure standard USB bulk and isochronous endpoints using the f_midi function
mkdir -p functions/midi.usb0
echo "1" > functions/midi.usb0/in_ports
echo "1" > functions/midi.usb0/out_ports

# Bind function to configuration
ln -s functions/midi.usb0 configs/c.1/

# Enable OS descriptors for the configuration
ln -s configs/c.1 os_desc/c.1

# Bind to the physical controller
UDC_CONTROLLER=$(ls /sys/class/udc | head -n 1)
if [ -z "${UDC_CONTROLLER}" ]; then
  echo "Error: No physical USB device controller found (UDC). Ensure dwc2 is loaded."
  exit 1
fi

echo "Binding gadget to controller: ${UDC_CONTROLLER}"
echo "${UDC_CONTROLLER}" > UDC
echo "OMID USB Gadget setup completed successfully!"
