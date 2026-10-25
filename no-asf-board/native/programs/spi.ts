// SPI receive through the native runtime: the board's default SPI (MOSI PB23,
// MISO PB22, SCK PA17 on SERCOM1). A shift-register device answers each byte
// with the one it received before; the program reports what it read back.
const first = pins.spiWrite(0x5a)
const second = pins.spiWrite(0xa5)
forever(() => {
    console.log("BW-SPI rx=" + first + "," + second)
    pause(100)
})
