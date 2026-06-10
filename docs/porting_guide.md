# OMID Embedded Porting & Integration Guide

This document provides guide parameters and code templates for porting the OMID Protocol to bare-metal microcontrollers or Real-Time Operating Systems (RTOS) like Zephyr or FreeRTOS.

---

## 1. Minimal C11 Packet Specification

In embedded C environments, the 8-byte OMID packet is structured using a packed structure definition:

```c
#include <stdint.h>

#pragma pack(push, 1)
typedef struct {
    uint16_t object_id;    // Unique fader/key ID
    uint8_t  event_type;   // Mapped to EventType enum
    uint8_t  flags;        // Bit 0: Touched, Bit 1: Raw, Bit 2: Dir, Bit 3-7: Subsample Offset
    uint32_t payload;      // Value (f32, i32, or raw ADC)
} OmidPacket;
#pragma pack(pop)

// Example helper to serialize
static inline void omid_serialize(const OmidPacket* packet, uint8_t* out_buf) {
    __builtin_memcpy(out_buf, packet, sizeof(OmidPacket));
}
```

---

## 2. FreeRTOS Task Integration

OMID control queues can be modeled using FreeRTOS `QueueHandle_t`. High-priority DSP threads read from these queues while ISRs populate them.

```c
#include "FreeRTOS.h"
#include "queue.h"
#include "task.h"

#define OMID_QUEUE_LEN 256

QueueHandle_t xOmidQueue;

void vMidiAdcReaderTask(void *pvParameters) {
    xOmidQueue = xQueueCreate(OMID_QUEUE_LEN, sizeof(OmidPacket));
    
    while (1) {
        uint16_t adc_val = read_hardware_adc(0); // Fader ADC
        
        OmidPacket packet = {
            .object_id = 0,
            .event_type = 0x01, // AbsoluteChange
            .flags = 0x02,       // Raw Data flag
            .payload = adc_val
        };
        
        xQueueSend(xOmidQueue, &packet, portMAX_DELAY);
        vTaskDelay(pdMS_TO_TICKS(1)); // 1ms polling intervals
    }
}
```

---

## 3. Zephyr RTOS Driver Binding

For Zephyr RTOS, devices expose OMID using standard Device Tree overlays and FIFO bindings:

```c
#include <zephyr/kernel.h>
#include <zephyr/sys/slist.h>

K_FIFO_DEFINE(omid_fifo);

struct omid_fifo_item {
    sys_snode_t node;
    OmidPacket packet;
};

void omid_submit_from_isr(OmidPacket *pkt) {
    struct omid_fifo_item *item = k_malloc(sizeof(struct omid_fifo_item));
    if (item != NULL) {
        item->packet = *pkt;
        k_fifo_put(&omid_fifo, item);
    }
}
```

---

## 4. UACT Audio Interleaving via I2S DMA

Under the Unified Audio & Control Transport (UACT), audio PCM buffers and OMID control envelopes are multiplexed.

On microcontrollers with I2S (Inter-IC Sound) DMA, the hardware is configured to transfer blocks of samples.
For an $N$-channel configuration, the DMA buffer layout is structured as:

$$\underbrace{A_0, A_1, \dots, A_{N-1}}_{\text{PCM Audio Channel Samples (f32/i32)}} , \underbrace{C_0, C_1}_{\text{8-Byte OMID Control Packet (split into 2 words)}}$$

### Implementation Strategy
1. **DMA Double Buffering**: Set up I2S DMA in ping-pong circular buffer mode.
2. **Interrupt Callback**: In the half-transfer and transfer-complete interrupts, parse the last 8 bytes of the active buffer block as the `control` field, and the preceding bytes as continuous PCM samples.
3. **Timer Sync**: Read the hardware timer counter register during the key change ISR and copy the lower 5 bits of the delta into the OMID flags field to ensure sub-sample accurate synchronization.
