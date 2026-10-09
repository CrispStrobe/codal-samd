# Authored ARMv7E-M foundation inventory

The separately named `armv7em-authored-foundations` profile binds eight owned
source/header/startup/linker files to immutable own-fork source
`2a991760339fe89f5272f923baf89dc18fd434d0`, descended from historical consumer
base `5bd6b93c219c7e784e885ba2d6812809fb6289a8`. This is an explicit inventory
choice for the existing prototypes, not adoption into current `master`, the
historical whole image or a consumer package. Existing runtime CMake remains
unchanged and still selects ASF inputs; this inventory does not invoke it.

Run `python3 tools/foundation-profile/check_profile.py`. Successful output is
only `SOURCE_PROFILE_ONLY`. The validator accepts exactly the three authored
fixtures and their exact original bytes, rejects extra/missing files, changed
hashes, changed source/base, duplicate JSON keys, runtime admission/build claims
and unsupported requested services. `--require native-arcade` must refuse.
It neither builds nor executes a guest, chooses compiler flags, discovers a
complete dependency graph nor certifies licensing or an image. It must not be
used as the consumer firmware admission gate.

## Selected APIs and consumers

| Header | API boundary | Selected authored consumers |
| --- | --- | --- |
| `simulation/include/InterruptMask.h` | Privileged same-core ARMv7E-M Thumb `Saved`, `read`, `saveAndDisable`, `restore`; reverse-order restoration, no host fallback or DMA barrier. | `interrupt-mask-probe.cpp`, `irq-mask-probe.cpp`; both have dedicated startup/vector contracts. |
| `simulation/include/SpiTransferState.h` | Noncopyable C++11 object; uint64 token; `begin`, `dmaComplete`, `txComplete`, `dmaError`, `cancel`, `timeout`, `take`; distinct success/error/cancel/timeout result. | `probe.cpp` only; events are abstract authored calls, not observed hardware DMA/TXC or callbacks. |

Paths and SHA256 digests are in [the machine-readable inventory](../profiles/armv7em-foundations.json).
These are selected fixture consumers, **not** the full original CODAL API/ABI
or transitive consumer closure. Standard C++ headers/toolchain contributions,
device definitions, clocks/cache, assembly, original public field/type users,
generated configuration and every real runtime consumer remain separately
reviewable inputs. Preserve inherited notices; reviewed new foundation files
carry MIT notices, not blanket clearance for vendor directories.

## Evidence and next change

Engine controls landed in [LabWired PR189](https://github.com/CrispStrobe/labwired-core/pull/189),
[PR190](https://github.com/CrispStrobe/labwired-core/pull/190) and
[PR191](https://github.com/CrispStrobe/labwired-core/pull/191). Their fixture
pins and observed boundaries remain authoritative; inventory validation is
not a new guest run or replacement for their independent original audits.
No fixture bytes changed here and no unchanged ARM qualification is replayed.

Next enumerate the actual selected CODAL API/header/source/consumer closure
and introduce a separately qualified build selector without ASF fallback.
Clean-build every affected consumer on hosted CI, fail on unsupported required
services, then integrate critical sections through actual callers before
peripheral IRQ/DMA/fiber work. Follow the
[runtime replacement plan](https://github.com/CrispStrobe/labwired-core/blob/main/docs/engineering/samd-runtime-replacement-plan.md).
No GPIO, polling SPI, UART, ADC/I2C, DMA, USB, QSPI, audio, native Arcade,
whole-runtime admission, physical safety, app pin, store or RTx claim follows.
