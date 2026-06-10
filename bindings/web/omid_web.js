/**
 * OMID WebUSB / WebMIDI Browser Library
 *
 * Exposes methods to search for, connect to, read, and write
 * raw OMID packets directly from modern web browsers.
 */

export class OmidWebUsbDevice {
  constructor() {
    this.device = null;
    this.interfaceNumber = 0;
    this.endpointIn = 1;
    this.endpointOut = 1;
    this.onPacketCallback = null;
    this.running = false;
  }

  /**
   * Requests permission and connects to the physical OMID device.
   */
  async requestDevice() {
    // Linux Foundation test vendor ID / ConfigFS gadget IDs
    this.device = await navigator.usb.requestDevice({
      filters: [{ vendorId: 0x1d6b, productId: 0x0104 }]
    });

    await this.device.open();
    await this.device.selectConfiguration(1);
    await this.device.claimInterface(this.interfaceNumber);
    console.log("Connected to OMID WebUSB device:", this.device.productName);
  }

  /**
   * Starts the read loop for incoming OMID packets.
   */
  async startReading() {
    this.running = true;
    while (this.running) {
      try {
        // Read exactly 8 bytes (size of OmidPacket)
        const result = await this.device.transferIn(this.endpointIn, 8);
        if (result.status === 'ok' && result.data.byteLength === 8) {
          const packet = this.parsePacket(result.data);
          if (this.onPacketCallback) {
            this.onPacketCallback(packet);
          }
        }
      } catch (err) {
        console.error("WebUSB transfer error:", err);
        await new Promise(r => setTimeout(r, 100)); // rate limit retry
      }
    }
  }

  /**
   * Stops the read loop.
   */
  stopReading() {
    this.running = false;
  }

  /**
   * Sends an 8-byte OMID packet to the hardware.
   * @param {Object} packet
   */
  async sendPacket(packet) {
    if (!this.device) throw new Error("Device not connected");
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);

    view.setUint16(0, packet.objectId, true);
    view.setUint8(2, packet.eventType);
    view.setUint8(3, packet.flags);
    view.setUint32(4, packet.payload, true);

    await this.device.transferOut(this.endpointOut, buffer);
  }

  /**
   * Registers a callback for incoming parsed packets.
   */
  onPacket(callback) {
    this.onPacketCallback = callback;
  }

  /**
   * Parses 8 bytes into an OMID packet object.
   * @param {DataView} dataView
   */
  parsePacket(dataView) {
    const view = new DataView(dataView.buffer);
    const objectId = view.getUint16(0, true);
    const eventType = view.getUint8(2);
    const flags = view.getUint8(3);
    const payload = view.getUint32(4, true);

    // Parse flags
    const touched = (flags & 0x01) !== 0;
    const rawData = (flags & 0x02) !== 0;
    const direction = (flags & 0x04) !== 0;
    const timerDelta = (flags & 0xF8) >> 3;

    return {
      objectId,
      eventType,
      flags: { touched, rawData, direction, timerDelta },
      payload,
      get f32() {
        return view.getFloat32(4, true);
      },
      get i32() {
        return view.getInt32(4, true);
      },
      get adc12() {
        return payload & 0x0FFF;
      },
      get adc16() {
        return payload & 0xFFFF;
      }
    };
  }
}
