// SPDX-License-Identifier: MIT
// Copyright (c) 2026 CrispStrobe contributors
#ifndef BRICKWRIGHT_SIMULATION_PAIRED_IRQ_STATE_H
#define BRICKWRIGHT_SIMULATION_PAIRED_IRQ_STATE_H

namespace brickwright_simulation {

// Pure policy. The adapter must atomically save-and-mask before EVERY call,
// then apply the returned mask. Not a scheduler, IRQ primitive or DMA barrier.
template<unsigned MaxDepth = 64>
class PairedIrqState {
    static_assert(MaxDepth > 0, "positive bounded critical-section depth required");
public:
    struct Owner {
        unsigned exception;
        unsigned fiber;
        bool valid() const {
            // Thread mode requires an actual stable nonzero scheduler identity.
            // NMI/HardFault cannot be serialized by PRIMASK and are unsupported.
            const bool configurable = exception == 4 || exception == 5 || exception == 6 ||
                                      exception == 11 || exception == 12 || exception == 14 || exception == 15;
            return (exception == 0 && fiber != 0) ||
                   ((configurable || (exception >= 16 && exception <= 255)) && fiber == 0);
        }
        bool equals(Owner other) const {
            return exception == other.exception && fiber == other.fiber;
        }
    };
    enum class Fault : unsigned {
        none = 0, invalid_mask = 1, invalid_owner = 2, underflow = 3,
        depth_limit = 4, owner_changed = 5, mask_lost = 6
    };
    struct Transition {
        Fault fault;
        unsigned restoreMask;
        bool ok() const { return fault == Fault::none; }
    };

    constexpr PairedIrqState() = default;
    PairedIrqState(const PairedIrqState&) = delete;
    PairedIrqState& operator=(const PairedIrqState&) = delete;

    Transition enter(unsigned savedMask, Owner owner) {
        if (fault_ != Fault::none) return {fault_, 1};
        if (savedMask > 1) return reject(Fault::invalid_mask);
        if (!owner.valid()) return reject(Fault::invalid_owner);
        if (depth_ != 0) {
            if (!owner_.equals(owner)) return reject(Fault::owner_changed);
            if (savedMask != 1) return reject(Fault::mask_lost);
        }
        if (depth_ == MaxDepth) return reject(Fault::depth_limit);
        if (depth_ == 0) {
            outerMask_ = savedMask;
            owner_ = owner;
        }
        ++depth_;
        return {Fault::none, 1};
    }

    Transition leave(unsigned savedMask, Owner owner) {
        if (fault_ != Fault::none) return {fault_, 1};
        if (savedMask > 1) return reject(Fault::invalid_mask);
        if (!owner.valid()) return reject(Fault::invalid_owner);
        if (depth_ == 0) return reject(Fault::underflow);
        if (!owner_.equals(owner)) return reject(Fault::owner_changed);
        if (savedMask != 1) return reject(Fault::mask_lost);
        --depth_;
        return {Fault::none, depth_ == 0 ? outerMask_ : 1};
    }

    unsigned depth() const { return depth_; }
    Fault fault() const { return fault_; }
    bool maySwitchContext() const { return depth_ == 0 && fault_ == Fault::none; }

private:
    unsigned depth_ = 0;
    unsigned outerMask_ = 1;
    Owner owner_ = {0, 0};
    Fault fault_ = Fault::none;
    Transition reject(Fault fault) {
        fault_ = fault; // Terminal for this lifetime: no reset/recovery API.
        return {fault_, 1};
    }
};
}
#endif
