// Audio through the native runtime: one tone through the mixer to the DAC on
// PA02 (codal SAMDDAC: TC3 paces DMA half-word samples into DAC DATA0).
music.playTone(440, 200)
forever(() => {
    console.log("BW-AUDIO played")
    pause(100)
})
