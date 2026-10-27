// Settings deletion and bank compaction. Each boot reports what it reads, then
// takes one step: seed "bw-keep" and "bw-a"; remove "bw-a"; rewrite the
// 500-byte "bw-gc" forty times, which fills the 16 KiB bank and compacts it.
const a = settings.readNumber("bw-a")
const keep = settings.readNumber("bw-keep")
const gc = settings.readBuffer("bw-gc")
control.dmesg("BW-STORE read=" + a + " keep=" + keep + " gc=" + (gc ? gc[0] : -1) + " ")
if (keep === undefined) {
    settings.writeNumber("bw-keep", 7)
    settings.writeNumber("bw-a", 41)
    control.dmesg("BW-STORE wrote=seed ")
} else if (a !== undefined) {
    settings.remove("bw-a")
    control.dmesg("BW-STORE wrote=removed ")
} else {
    const buf = control.createBuffer(500)
    for (let i = 1; i <= 40; i++) {
        buf.fill(i)
        settings.writeBuffer("bw-gc", buf)
    }
    control.dmesg("BW-STORE wrote=gc ")
}
