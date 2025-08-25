use std::io::{self, Read, Write};
use std::process::ExitCode;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::Duration;

use clap::{Parser, Subcommand};
use stopwatch_rust::{format_duration, Stopwatch, StopwatchError, StopwatchErrorKind};

#[derive(Parser)]
#[command(name = "stopwatch", version = concat!("v", env!("CARGO_PKG_VERSION")), about = "Stopwatch CLI (hybrid: REPL + batch) with realtime watch")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// (Opsional) Kompat: jalankan perintah langsung tanpa subcommand `run`.
    #[arg(trailing_var_arg = true)]
    legacy_cmds: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Batch mode: eksekusi beruntun, contoh: `stopwatch run start elapsed stop`
    Run { cmds: Vec<String> },
    /// REPL interaktif eksplisit (tanpa argumen juga masuk REPL)
    Interactive,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    // Subcommand eksplisit
    if let Some(Commands::Run { cmds }) = &cli.command {
        return run_batch(cmds.clone());
    }
    if matches!(cli.command, Some(Commands::Interactive)) {
        return run_repl();
    }

    // Kompat-lama: argumen langsung tanpa subcommand
    if !cli.legacy_cmds.is_empty() {
        return run_batch(cli.legacy_cmds.clone());
    }

    // Default: REPL
    run_repl()
}

fn run_batch(cmds: Vec<String>) -> ExitCode {
    if cmds.is_empty() {
        eprintln!("error: no commands. Use `stopwatch -h` for help.");
        return ExitCode::from(2);
    }
    let mut sw = Stopwatch::new();
    for cmd in cmds {
        if let Err(e) = dispatch(&mut sw, &cmd) {
            eprintln!("error: {:?} (cmd: {cmd})", e);
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}

fn run_repl() -> ExitCode {
    println!("Stopwatch REPL. Perintah: start | stop | reset | elapsed | watch | help | exit");
    let mut sw = Stopwatch::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }
        let cmd = line.trim();
        if matches!(cmd, "exit" | "quit") {
            break;
        }
        if cmd.eq_ignore_ascii_case("help") {
            print_help();
            continue;
        }
        if let Err(e) = dispatch(&mut sw, cmd) {
            eprintln!("error: {:?} (cmd: {cmd})", e);
        }
    }
    ExitCode::SUCCESS
}

fn dispatch(sw: &mut Stopwatch, cmd: &str) -> Result<(), StopwatchError> {
    match cmd {
        "start" => sw.start(),
        "stop" => sw.stop(),
        "reset" => {
            sw.reset();
            Ok(())
        }
        "elapsed" => {
            println!("{}", format_duration(sw.elapsed()));
            Ok(())
        }
        "watch" => {
            run_watch(sw);
            Ok(())
        }
        "-h" | "--help" => {
            print_help();
            Ok(())
        }
        "-V" | "--version" => {
            println!("stopwatch-rust v{}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        _ => Err(StopwatchError(StopwatchErrorKind::Invalid)),
    }
}

fn run_watch(sw: &mut Stopwatch) {
    // Jika belum berjalan, otomatis mulai (abaikan AlreadyRunning)
    let _ = sw.start();

    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_reader = stop_flag.clone();

    // Thread pembaca: hentikan watch saat pengguna menekan Enter
    thread::spawn(move || {
        let mut buf = String::new();
        let _ = io::stdin().read_line(&mut buf);
        stop_reader.store(true, Ordering::SeqCst);
    });

    // Sembunyikan kursor selama watch (ANSI), tampilkan petunjuk, dan kembalikan saat selesai
    print!("[Press Enter to switch command]\n\x1b[?25l");
    let _ = io::stdout().flush();

    while !stop_flag.load(Ordering::SeqCst) {
        let d = sw.elapsed();
        print!("\r{}", format_duration(d));
        let _ = io::stdout().flush();
        thread::sleep(Duration::from_millis(100));
    }

    print!("\r{}\n\x1b[?25h", format_duration(sw.elapsed()));
    let _ = io::stdout().flush();
}

fn print_help() {
    println!(
        "COMMANDS:\n  start        Mulai stopwatch\n  stop         Hentikan stopwatch & akumulasi waktu\n  reset        Setel ulang ke 00:00:00.000\n  elapsed      Cetak waktu kumulatif\n  watch        Tampilkan waktu berjalan realtime (tekan Enter untuk keluar)\n  help         Bantuan (REPL)\n  exit/quit    Keluar (REPL)\n\nMODES:\n  stopwatch run <cmds...>      # batch (exit code tegas)\n  stopwatch interactive        # REPL eksplisit\n  stopwatch <cmds...>          # kompat-lama (tanpa subcommand)\n  stopwatch                    # REPL default\n"
    );
}
