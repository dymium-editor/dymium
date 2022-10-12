# `dymium-term` -- Architecture

This crate is responsible for a few different things. Broadly speaking, it serves the following
functions:

1. Acts as a database for terminal capabilities
   * Default data stored in `capdata.yaml`, try:
     ```sh
     cargo r --bin verify-caps -- capdata.yaml
     ```
   * Largely implemented in `src/capinfo.rs`
2. Provides mid-level terminal commands (`dymium-term` can choose which escape sequence a command
   should map to, but that's it)
   * Top-level implementation comes from `src/cmd.rs`, pulling in e.g., `src/color` and
     `src/style.rs`.

Plus, eventually:

3. Contains the necessary pieces to parse CSI sequences, for implementing a terminal emulator on top
   of it.
