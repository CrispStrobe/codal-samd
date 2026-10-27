// Levels and mixing through the mixer to the DAC on PA02: A4 at volume 255,
// then at 64; then A4 and E5 together at 128 (within range), then together at
// 255 (beyond the mixer's range, so the sum clips).
music.setVolume(255)
music.playTone(440, 120)
music.setVolume(64)
music.playTone(440, 120)
music.setVolume(128)
control.runInParallel(() => music.playTone(440, 240))
music.playTone(660, 240)
music.setVolume(255)
control.runInParallel(() => music.playTone(440, 240))
music.playTone(660, 240)
forever(() => {
    console.log("BW-MIX played")
    pause(100)
})
