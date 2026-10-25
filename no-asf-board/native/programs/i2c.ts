// I2C master through the native runtime: register-pointer write, repeated
// start, read. The host attaches a BMP280 model at 0x77 (chip id 0x58 at 0xD0)
// and nothing at 0x10. Results go to the host over HF2.
pins.i2cWriteNumber(0x77, 0xD0, NumberFormat.UInt8LE, true)
const id = pins.i2cReadNumber(0x77, NumberFormat.UInt8LE, false)
const absent = pins.i2cWriteBuffer(0x10, pins.createBuffer(1), false)
forever(() => {
    console.log("BW-I2C id=" + id + " absent=" + absent)
    pause(100)
})
