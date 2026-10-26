// The whole palette through the native runtime: sixteen 10-pixel bands, colour
// 0 at screen x 0..10 up to colour 15 at x 150..160, over the full height.
const bands = image.create(160, 120)
for (let c = 0; c < 16; c++) {
    bands.fillRect(c * 10, 0, 10, 120, c)
}
scene.setBackgroundImage(bands)
forever(() => {
    console.log("BW-PALETTE ready")
    pause(100)
})
