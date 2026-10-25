// Display pixels through the native runtime: a green background and one 16x16
// red sprite whose top-left corner is at screen (32, 22).
scene.setBackgroundColor(7)
const block = image.create(16, 16)
block.fill(2)
const sprite = sprites.create(block)
sprite.setPosition(40, 30)
forever(() => {
    console.log("BW-FRAME ready")
    pause(100)
})
