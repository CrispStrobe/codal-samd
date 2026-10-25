// SPDX-License-Identifier: MIT
// TCC timer leaf: CODAL's SAMDTCCTimer on TCC0 of the no-ASF board, linked from
// the same compiled sources as the full-board base with this main instead of
// the MakeCode runtime. MakeCode cannot select a TCC system timer on SAMD
// (pxt's allocateTimer constructs TCC entries as TCs), so the driver is run
// directly: eight compare-channel-0 matches, 1000 us apart at the driver's
// 1 MHz counter clock, each re-armed from the interrupt.
#include "SAMDTCCTimer.h"

using namespace codal;

extern "C" {
volatile uint32_t bw_tcc_matches;
volatile uint32_t bw_tcc_counts[8];
volatile uint32_t bw_tcc_done;
}

static SAMDTCCTimer *timer;

static void on_match(uint16_t channels)
{
    if (!(channels & 1))
        return;
    uint32_t n = bw_tcc_matches;
    if (n < 8)
        bw_tcc_counts[n] = timer->captureCounter();
    bw_tcc_matches = n + 1;
    timer->offsetCompare(0, 1000);
}

int main()
{
    static SAMDTCCTimer tcc0(TCC0, TCC0_0_IRQn);
    timer = &tcc0;
    timer->setIRQ(on_match);
    timer->setCompare(0, 1000);
    timer->enable();
    while (bw_tcc_matches < 8)
        __WFI();
    timer->disable();
    bw_tcc_done = 1;
    for (;;)
        __WFI();
}
