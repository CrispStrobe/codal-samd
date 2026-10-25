// USART through the native runtime: the board's serial pins (TX PB16, RX PB17,
// SERCOM5). The program transmits one line, polls the receiver, and reports
// over HF2 what the host sent back. (onDelimiterReceived alone never enables
// the receiver: codal's Serial initialises RX on the first read.)
serial.writeLine("BW-UART tx")
let rx = ""
forever(() => {
    rx += serial.readString()
    const line = rx.split("\n")[0]
    console.log("BW-UART rx=" + (rx.indexOf("\n") >= 0 ? line : "-"))
    pause(100)
})
