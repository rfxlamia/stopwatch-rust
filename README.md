# Timer CLI (Rust)

A reliable command-line timer with hybrid modes, realtime watch, laps, export, and measure.

## Features

* Start, stop, reset, and show elapsed time
* Laps: record splits with optional labels; view deltas
* Export laps to JSON or CSV (stdout; pipe-friendly)
* Measure external commands and propagate exit code
* Interactive REPL (default), batch via `run`, legacy batch compatible
* Realtime `watch` mode (updates in-place; press Enter to exit)

## Clone & Setup

```bash
git clone https://github.com/rfxlamia/timer-cli.git
cd timer-cli
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

REPL commands: `start | stop | reset | elapsed | watch | lap [label] | laps | export [json|csv] | measure -- <cmd...> | help | exit`

### Batch mode (once per run)

```bash
cargo run -- run start lap setup lap coding stop elapsed laps export json
```

### Legacy batch mode (compatibility)

```bash
cargo run -- start lap build stop laps export csv
```

## Commands

* `start`: Start the timer
* `stop`: Stop the timer and accumulate elapsed time
* `reset`: Reset elapsed time to zero, stop, and clear laps
* `elapsed`: Print the current elapsed time
* `watch`: Show realtime elapsed time; press Enter to exit (screen shows: "Press Enter to switch command")
* `lap [label]`: Record a lap at current elapsed time
* `laps`: Show lap table with deltas
* `export [json|csv]`: Print laps to stdout (pipe to files)
* `measure -- <cmd...>`: Run an external command and report duration; propagates exit code
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
