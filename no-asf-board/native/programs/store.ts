// Settings storage: the counter read at startup, then written back plus one.
// The runtime log carries both reports, so they are visible before USB starts.
const before = settings.readNumber("bw-count")
control.dmesg("BW-STORE read=" + before)
const after = (before || 40) + 1
settings.writeNumber("bw-count", after)
control.dmesg("BW-STORE wrote=" + after)
forever(() => {
    console.log("BW-STORE idle")
    pause(100)
})
