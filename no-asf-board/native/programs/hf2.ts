// HF2 debug services through the native runtime: the host finds this global
// through DBG_GET_GLOBAL_STATE, rewrites it with WRITE_WORDS, and the program
// reports the new value.
let marker = 1234
forever(() => {
    console.log("BW-HF2 marker=" + marker)
    pause(100)
})
