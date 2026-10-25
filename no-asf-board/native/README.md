# Native MakeCode program on the no-ASF board

An empty public `main.ts`, compiled natively by the admitted Arcade 4.2.1 /
PXT 13.2.1 compiler onto the [full-board base](../../docs/PXT-NO-ASF-FULLBOARD-BASE.md),
boots in LabWired and answers HF2 over USB.

## Image

[boot.py](boot.py) assembles one flash image from the compiler's UF2 output.
Below the application origin 0x4000 there is no bootloader code: an authored
8-byte reset forward (the program's own stack top and reset entry) and the
admitted [CF2 fragments](../../docs/PXT-CF2-AUTHORED-METADATA.md): configuration
words at 0x3000, the information string at 0x3400 and the 16-byte UF2
information block at 0x3ff0 (`config_data`, two absent handover hooks,
`info_uf2`). This matches the layout PXT reads through
`UF2_BINFO = BOOTLOADER_END - sizeof(UF2_BInfo)`.

## Case

[guest.rs](guest.rs) runs in LabWired with the PyBadge chip descriptor and the
opt-in board models in [peripherals.yaml](peripherals.yaml): SAMD51 GCLK,
OSCCTRL, TC0–TC5, CMCC, NVMCTRL, the USB device banks and the SERCOM4-TX DMAC.
Declared authored NVM inputs supply the SW0 calibration word and a serial
number; they are not hardware captures.

- **Boot**: architectural reset from address 0 through the reset forward,
  data/BSS, clock tree, runtime constructors, `pxt::start`, `initRuntime`
  (system timer on TC0 with delivered interrupts, CF2 reads, NeoPixel and display
  SPI initialisation) and the scheduler. After the program's 500 ms
  `start_usb` sleep the USB controller attaches.
- **Enumeration**: the host resets the bus after the controller attaches and
  enumerates once the real reset handler has returned. The device descriptor
  must equal PXT's source table; the configuration must describe the HF2
  interface (vendor class, subclass 42, bulk EP1 IN / EP2 OUT) and the
  endpoint-less dummy interface.
- **HF2**: `BININFO` must report user-space mode, a 260-byte message size and
  family `0x55114460`; `INFO` must return exactly the authored information
  string, which the guest finds through the 0x3ffc pointer.

Every reply is produced by guest code through the USB model; nothing is
synthesised. Manufactured controls mutate only the input image:
`no-info-pointer` clears `info_uf2`, and `INFO` must return the guest's own
`"N/A"` fallback; `erased-reset-forward` erases the reset words, and the CPU
must fail at its first fetch without entering `Reset_Handler`.

## Limits

Step counts are guest instructions, not time. No display output, input,
audio, debugger, flashing over HF2, physical timing or complete peripheral
reset is qualified. The DMAC is the SERCOM4 transmit profile only.
