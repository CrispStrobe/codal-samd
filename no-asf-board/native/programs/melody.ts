// Two notes and a rest through the mixer to the DAC on PA02: A4 for 150 ms,
// then A5 for 150 ms, then silence.
music.playTone(440, 150)
music.playTone(880, 150)
forever(() => {
    console.log("BW-MELODY played")
    pause(100)
})
