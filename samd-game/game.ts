// SPDX-License-Identifier: MIT
// Authored small paddle game: eight targets, bouncing ball, player movement,
// overlap callbacks, score/status and an A-button colour/audio response.
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
