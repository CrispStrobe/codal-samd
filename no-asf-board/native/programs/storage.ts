// Settings deletion, bank compaction and clearing. Each boot reports what it
// reads, then takes one step: seed "bw-keep" and "bw-a"; remove "bw-a";
// rewrite the 500-byte "bw-gc" forty times, which fills the 16 KiB bank and
// compacts it; forty times more, which compacts it back; clear all settings.
const a = settings.readNumber("bw-a")
const keep = settings.readNumber("bw-keep")
const gc = settings.readBuffer("bw-gc")
const last = gc ? gc[0] : -1
control.dmesg("BW-STORE read=" + a + " keep=" + keep + " gc=" + last + " ")
function rewrite(from: number) {
    const buf = control.createBuffer(500)
    for (let i = from; i < from + 40; i++) {
        buf.fill(i)
        settings.writeBuffer("bw-gc", buf)
    }
}
if (keep === undefined) {
    settings.writeNumber("bw-keep", 7)
    settings.writeNumber("bw-a", 41)
    control.dmesg("BW-STORE wrote=seed ")
} else if (a !== undefined) {
    settings.remove("bw-a")
    control.dmesg("BW-STORE wrote=removed ")
} else if (last < 40) {
    rewrite(1)
    control.dmesg("BW-STORE wrote=gc ")
} else if (last < 80) {
    rewrite(41)
    control.dmesg("BW-STORE wrote=gc2 ")
} else {
    settings.clear()
    control.dmesg("BW-STORE wrote=cleared ")
}
