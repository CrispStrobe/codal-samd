// Buttons through the native runtime: the PyBadge reads its keys through a
// 74HC165 shift register (latch PB00, clock PB31, data PB30).
let aPresses = 0
controller.A.onEvent(ControllerButtonEvent.Pressed, () => {
    aPresses++
})
forever(() => {
    console.log("BW-INPUT a=" + aPresses + " left=" + (controller.left.isPressed() ? 1 : 0) + " ah=" + (controller.A.isPressed() ? 1 : 0))
    pause(50)
})
