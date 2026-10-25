// PWM through the native runtime: the LED pad (PA23) as an analog output.
// codal's pwmout picks TC4 (WO1, PMUX E) for it; the host measures the pad.
const led = pins.pinByCfg(DAL.CFG_PIN_LED) as AnalogOutPin
led.analogWrite(512)
forever(() => {
    console.log("BW-PWM set")
    pause(100)
})
