# Stopwatch CLI (Rust)

A minimal command-line stopwatch with hybrid modes and realtime watch.

## Features

* Start, stop, reset, and show elapsed time
* Chain multiple commands in a single run (batch)
* Interactive REPL (default when no args)
* Realtime `watch` mode (updates in-place; press Enter to exit)
* Help (`-h`/`--help`) and version (`-V`/`--version`)

## Clone & Setup

```bash
git clone https://github.com/rfxlamia/stopwatch-rust.git
cd stopwatch-rust
cargo build
cargo run
```

## Build

```bash
cargo build
```

## Modes

### REPL (default)

Run without arguments:

```bash
cargo run
```

REPL commands: `start | stop | reset | elapsed | watch | help | exit`

### Batch mode (once per run)

```bash
cargo run -- run start elapsed stop elapsed
```

### Legacy batch mode (compatibility)

```bash
cargo run -- start elapsed stop elapsed
```

## Commands

* `start`: Start the stopwatch
* `stop`: Stop the stopwatch and accumulate elapsed time
* `reset`: Reset elapsed time to zero and stop
* `elapsed`: Print the current elapsed time
* `watch`: Show realtime elapsed time; press Enter to exit (screen shows: "Press Enter to switch command")
* `-h`, `--help`: Show help
* `-V`, `--version`: Show version

## Realtime `watch`

Display elapsed time on a single line. Press Enter to return to REPL.

```bash
cargo run
> start
> watch
00:00:12.345  # updating; press Enter to exit
> stop
> elapsed
```

## Parallel workflow example

1. Terminal A: `cargo run` → `start` (keep REPL running)
2. Terminal B: work on your project (build/test/etc.)
3. Terminal A: `watch` (press Enter to return) → `stop` → `elapsed`

## Output format

* Elapsed time prints as `HH:MM:SS.mmm`

## Exit codes (batch)

* `0`: success
* `1`: command/state error (e.g., stop before start, double start)
* `2`: no/invalid arguments

## State contract

* Starting while already running returns an error
* Stopping while not running returns an error
* Reset always sets time to `00:00:00.000` and not running

## Testing

```bash
cargo test
```

## Help & Version

```bash
cargo run -- --help
cargo run -- --version
```

## License

MIT — see [LICENSE.md](LICENSE.md).
