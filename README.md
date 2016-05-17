
A few notes as we go along.

* serial crate needs io flushing on input and output (to resync dirty connections).
* rscam crate needs single-frame capture.

Structural:

* commands are lHEX, rHEX, +HEX, -HEX
* 6400 (0x1900) steps in a full revolution
