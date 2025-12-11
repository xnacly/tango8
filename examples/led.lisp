; vim: filetype=racket
;
; One to one port of examples/led.t8
;
; Compile via: cargo run -p lisp examples/led.lisp
; Emulate via: cargo run -p emu examples/led.lisp.t8b

; constants are resolved at compile time
(const led 0xF)
(const off 0)
(const on 0)

; unrolled to 5 times 
; 
; LOADI 1
; ST [0xF]
; LOADI 0
; ST [0xF]
;
(repeat 5
    (write &led on)
    (write &led off))
