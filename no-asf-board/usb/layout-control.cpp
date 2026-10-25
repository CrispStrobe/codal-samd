// SPDX-License-Identifier: MIT
// Pure host layout/numeric control only: never dereference the MMIO pointers.
#define BW_USB_LAYOUT_CONTROL 1
#include "samd51_usb_mmio.h"
#include <assert.h>
#include <type_traits>
static_assert(std::is_same<decltype(&USB->DEVICE.DeviceEndpoint[0]), UsbDeviceEndpoint *>::value, "real driver endpoint pointer ABI");
static_assert(std::is_same<decltype(&USB->DEVICE.DeviceEndpoint[0].EPCFG.reg), volatile uint8_t *>::value, "MMIO register access remains volatile");
int main() {
    BwUsbPadCalibration pad={};
    pad.bit.TRANSN=9;pad.bit.TRANSP=25;pad.bit.TRIM=6;
    assert(pad.reg==0x6259);
    BwUsbPacketSize packet={};
    packet.bit.BYTE_COUNT=65;packet.bit.MULTI_PACKET_SIZE=64;packet.bit.SIZE=3;
    assert(packet.reg==((3u<<28)|(64u<<14)|65u));
    BwUsbAhbMask ahb={};ahb.bit.USB_=1;assert(ahb.reg==(1u<<10));
    BwUsbApbbMask apbb={};apbb.bit.USB_=1;assert(apbb.reg==1);
    BwUsbControl ctrl={};ctrl.bit.SWRST=1;assert(ctrl.reg==1);
    ctrl.reg=0;ctrl.bit.ENABLE=1;ctrl.bit.RUNSTDBY=1;assert(ctrl.reg==0x06); // RUNSTDBY is bit 2 (ATDF); bit 6 reserved
    BwUsbGclkSync sync={};sync.bit.GENCTRL0=1;assert(sync.reg==4);
    const uint32_t fuse=9u|(25u<<5)|(6u<<10);
    assert(((fuse&USB_FUSES_TRANSN_Msk)>>USB_FUSES_TRANSN_Pos)==9);
    assert(((fuse&USB_FUSES_TRANSP_Msk)>>USB_FUSES_TRANSP_Pos)==25);
    assert(((fuse&USB_FUSES_TRIM_Msk)>>USB_FUSES_TRIM_Pos)==6);
    assert(sizeof(BwUsbDevice)==512);
    return 0;
}
