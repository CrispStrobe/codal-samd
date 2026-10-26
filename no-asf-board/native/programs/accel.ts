// extensions: accelerometer
// The accelerometer through the native runtime: CODAL's LIS3DH driver on the
// board's I2C bus (CF2 ACCELEROMETER_TYPE 0x30, address 0x18). The host
// attaches a sensor holding a fixed reading; the program reports milli-g.
forever(() => {
    const x = input.acceleration(Dimension.X)
    const y = input.acceleration(Dimension.Y)
    const z = input.acceleration(Dimension.Z)
    console.log("BW-ACCEL x=" + x + " y=" + y + " z=" + z)
    pause(100)
})
