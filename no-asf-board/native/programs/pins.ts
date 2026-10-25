// Pins and external interrupts through the native runtime. D4 (PA14) drives,
// D5 (PA16) reads; the host jumpers PA14 to PA16. Results go to the host over HF2.
const drive = pins.pinByCfg(DAL.CFG_PIN_D4)
const sense = pins.pinByCfg(DAL.CFG_PIN_D5)
let rises = 0
sense.onEvent(PinEvent.Rise, () => { rises++ })
drive.digitalWrite(false)
pause(20)
const low = sense.digitalRead()
drive.digitalWrite(true)
pause(20)
const high = sense.digitalRead()
drive.digitalWrite(false)
pause(20)
drive.digitalWrite(true)
pause(20)
forever(() => {
    console.log("BW-PINS low=" + (low ? 1 : 0) + " high=" + (high ? 1 : 0) + " rises=" + rises)
    pause(100)
})
