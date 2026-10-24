// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
// Authored USB diagnostic application, using actual MIT core/SAMD driver code.
// Its vendor echo interface is not HF2, PXT native startup or a physical device.
#include "Runtime.h"
#include "CodalUSB.h"
#include "InterruptMask.h"
#ifndef BW_USB_LEAF_INTERRUPT
#define BW_USB_LEAF_INTERRUPT 0
#endif
#ifndef BW_USB_LEAF_HIDE_WEBUSB
#define BW_USB_LEAF_HIDE_WEBUSB 0
#endif
extern "C" {
volatile unsigned usb_leaf_stage=0, usb_leaf_initialised=0;
volatile unsigned usb_leaf_echo_checks=0, usb_leaf_echo_done=0;
volatile unsigned usb_leaf_data=0x1234abcd, usb_leaf_bss=0;
}
static void require(bool ok,unsigned reason) {
    if(!ok) target_panic(reason);
}
class LeafEcho final : public CodalUSBInterface {
public:
    const InterfaceInfo *getInterfaceInfo() override {
        static const InterfaceInfo info={
            nullptr,0,2,{2,0xff,0x42,1,0,0},
            {BW_USB_LEAF_INTERRUPT ? USB_EP_TYPE_INTERRUPT : USB_EP_TYPE_BULK,1},
            {BW_USB_LEAF_INTERRUPT ? USB_EP_TYPE_INTERRUPT : USB_EP_TYPE_BULK,1}};
        return &info;
    }
    bool enableWebUSB() override {return !BW_USB_LEAF_HIDE_WEBUSB;}
    int endpointRequest() override {
        if(!out) return DEVICE_OK;
        unsigned char data[64];
        int size=out->read(data,sizeof(data));
        if(!size) return DEVICE_OK;
        require(!usb_leaf_echo_done && size==64,0x711);++usb_leaf_echo_checks;
        for(unsigned i=0;i<64;++i) {
            require(data[i]==((i*37u+11u)&255u),0x712);++usb_leaf_echo_checks;
        }
        require(in && in->write(data,size)==size,0x713);++usb_leaf_echo_checks;
        usb_leaf_echo_done=1;
        return DEVICE_OK;
    }
};
void runtime_probe() {
    using Mask=brickwright_simulation::InterruptMask;
    require(Mask::read()==1 && usb_leaf_data==0x1234abcd && usb_leaf_bss==0,0x701);
    runtime_status=0x100;
    auto device=new CodalUSB();
    auto echo=new LeafEcho();
    require(device->add(*echo)==DEVICE_OK,0x703);
    usb_leaf_stage=1;
    require(device->start()==DEVICE_OK,0x702);
    usb_leaf_stage=2;
    Mask::restore(0);
    for (;;) {
        asm volatile(".global usb_leaf_wait\nusb_leaf_wait: wfi" ::: "memory");
        usb_leaf_initialised=device->isInitialised();
        if(usb_leaf_initialised) usb_leaf_stage=3;
        if(usb_leaf_initialised && usb_leaf_echo_done) {
            require(usb_leaf_echo_checks==66 && !Mask::read(),0x714);
            usb_leaf_stage=4;runtime_status=0x600d;
        }
    }
}
