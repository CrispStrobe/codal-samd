// SPDX-License-Identifier: MIT
// Authored small paddle game: eight targets, bouncing ball, player movement,
// overlap callbacks, score/status and an A-button colour/audio response.
// The physical GPIO shift-register descriptor is active-low. The admitted
// runtime reserves virtual pins1050..1057 for the inverted multiplexed inputs.
// Keep the original CF2/default1000..1007 profile unchanged; declare this
// game's complete native input contract in its compiled user configuration.
namespace userconfig {
    export const PIN_BTN_A = 1056
    export const PIN_BTN_B = 1057
    export const PIN_BTN_LEFT = 1050
    export const PIN_BTN_RIGHT = 1053
    export const PIN_BTN_UP = 1051
    export const PIN_BTN_DOWN = 1052
    export const PIN_BTN_MENU = 1054
    export const PIN_BTN_SOFT_RESET = 1055
}
scene.setBackgroundColor(7)
info.setScore(0)
const paddleArt = image.create(16, 16)
paddleArt.fill(2)
const paddle = sprites.create(paddleArt, SpriteKind.Player)
paddle.setPosition(40, 100)
paddle.setStayInScreen(true)
controller.moveSprite(paddle, 60, 0)
for (let i = 0; i < 8; i++) {
    const art = image.create(8, 8)
    art.fill(4)
    const target = sprites.create(art, SpriteKind.Enemy)
    target.setPosition(20 + i * 16, 20)
}
const ballArt = image.create(8, 8)
ballArt.fill(8)
const ball = sprites.create(ballArt, SpriteKind.Projectile)
ball.setPosition(80, 60)
ball.setVelocity(45, 30)
ball.setBounceOnWall(true)
sprites.onOverlap(SpriteKind.Projectile, SpriteKind.Enemy, (hit, target) => {
    hit.vy = Math.abs(hit.vy)
    info.changeScoreBy(1)
})
sprites.onOverlap(SpriteKind.Projectile, SpriteKind.Player, (hit, player) => {
    hit.vy = -Math.abs(hit.vy)
})
controller.A.onEvent(ControllerButtonEvent.Pressed, () => {
    paddleArt.fill(9)
    music.playTone(440, 50)
})
controller.A.onEvent(ControllerButtonEvent.Released, () => {
    paddleArt.fill(2)
})
