// ADC through the native runtime: A1 (PA05, ADC0 AIN5) and A2 (PB08, ADC0
// AIN2) read as 10-bit values against VDDANA. The host drives the inputs.
const a1 = pins.pinByCfg(DAL.CFG_PIN_A1) as AnalogInPin
const a2 = pins.pinByCfg(DAL.CFG_PIN_A2) as AnalogInPin
forever(() => {
    console.log("BW-ADC a1=" + a1.analogRead() + " a2=" + a2.analogRead())
    pause(100)
})
