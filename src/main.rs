use std::io::{self, Write};
use std::process::{ExitCode, Command};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::Duration;

use clap::{Parser, Subcommand};
use timer_cli::{format_duration, Timer, TimerError, TimerErrorKind};

#[derive(Parser)]
#[command(name = "timer-cli", version = env!("CARGO_PKG_VERSION"), about = "Timer CLI: REPL + batch, watch, lap/export, measure")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// (Opsional) Kompat: jalankan perintah langsung tanpa subcommand `run`.
    #[arg(trailing_var_arg = true)]
    legacy_cmds: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Batch mode: eksekusi beruntun, contoh: `timer-cli run start elapsed stop`
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
        eprintln!("error: no commands. Use `timer-cli -h` for help.");
        return ExitCode::from(2);
    }
    let mut t = Timer::new();
    for cmd in cmds {
        if let Err(e) = dispatch(&mut t, &cmd) {
            eprintln!("error: {:?} (cmd: {cmd})", e);
            return ExitCode::from(1);
        }
    }
    ExitCode::SUCCESS
}

fn run_repl() -> ExitCode {
    println!("Timer REPL. Perintah: start|stop|reset|elapsed|watch|lap [label]|laps|export [json|csv]|measure -- <cmd...>|help|exit");
    let mut t = Timer::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        let _ = io::stdout().flush();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() {
            break;
        }
        let s = line.trim();
        if matches!(s, "exit" | "quit") {
            break;
        }
        if s.eq_ignore_ascii_case("help") {
            print_help();
            continue;
        }
        if let Err(e) = dispatch(&mut t, s) {
            eprintln!("error: {:?} (cmd: {s})", e);
        }
    }
    ExitCode::SUCCESS
}

fn dispatch(t: &mut Timer, input: &str) -> Result<(), TimerError> {
    let mut parts = input.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    match cmd {
        "start"   => t.start(),
        "stop"    => t.stop(),
        "reset"   => { t.reset(); Ok(()) }
        "elapsed" => { println!("{}", format_duration(t.elapsed())); Ok(()) }
        "watch"   => { run_watch(t); Ok(()) }
        "lap"     => { let label = parts.next().map(|s| s.to_string()); t.lap(label) }
        "laps"    => { print_laps(t); Ok(()) }
        "export"  => { let fmt = parts.next().unwrap_or("json"); export_laps(t, fmt)?; Ok(()) }
        "measure" => { let cmdline: Vec<String> = parts.map(|s| s.to_string()).collect(); measure_command(cmdline)?; Ok(()) }
        "-h" | "--help" => { print_help(); Ok(()) }
        "-V" | "--version" => { println!("timer-cli v{}", env!("CARGO_PKG_VERSION")); Ok(()) }
        _ => Err(TimerError(TimerErrorKind::Invalid)),
    }
}

fn run_watch(t: &mut Timer) {
    // Jika belum berjalan, otomatis mulai (abaikan AlreadyRunning)
    let _ = t.start();

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
        let d = t.elapsed();
        print!("\r{}", format_duration(d));
        let _ = io::stdout().flush();
        thread::sleep(Duration::from_millis(100));
    }

    print!("\r{}\n\x1b[?25h", format_duration(t.elapsed()));
    let _ = io::stdout().flush();
}

fn print_laps(t: &Timer) {
    if t.laps().is_empty() {
        println!("(no laps)");
        return;
    }
    println!("#  time          delta        label");
    let mut prev_ms: u128 = 0;
    for lap in t.laps() {
        let delta_ms = lap.at_ms.saturating_sub(prev_ms);
        let at = Duration::from_millis(lap.at_ms as u64);
        let delta = Duration::from_millis(delta_ms as u64);
        println!(
            "{:<2} {:<12} {:<12} {}",
            lap.index,
            format_duration(at),
            format_duration(delta),
            lap.label.clone().unwrap_or_default()
        );
        prev_ms = lap.at_ms;
    }
}

fn export_laps(t: &Timer, fmt: &str) -> Result<(), TimerError> {
    match fmt {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&t.laps()).unwrap());
            Ok(())
        }
        "csv" => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.write_record(["index", "time_ms", "label"]).unwrap();
            for lap in t.laps() {
                wtr.write_record([
                    lap.index.to_string(),
                    lap.at_ms.to_string(),
                    lap.label.clone().unwrap_or_default(),
                ]).unwrap();
            }
            wtr.flush().unwrap();
            Ok(())
        }
        _ => Err(TimerError(TimerErrorKind::Invalid)),
    }
}

fn measure_command(cmdline: Vec<String>) -> Result<(), TimerError> {
    if cmdline.is_empty() { return Err(TimerError(TimerErrorKind::Invalid)); }
    let mut t = Timer::new();
    t.start()?;
    let status = Command::new(&cmdline[0]).args(&cmdline[1..]).status()
        .map_err(|_| TimerError(TimerErrorKind::Invalid))?;
    t.stop()?;
    println!("{} took {}", cmdline.join(" "), format_duration(t.elapsed()));
    std::process::exit(status.code().unwrap_or(1));
}

fn print_help() {
    println!(
        "COMMANDS:\n  start                Mulai timer\n  stop                 Hentikan timer & akumulasi waktu\n  reset                Setel ulang ke 00:00:00.000 (hapus laps)\n  elapsed              Cetak waktu kumulatif\n  watch                Tampilkan waktu realtime (Enter untuk kembali)\n  lap [label]          Tambah lap (hanya saat running)\n  laps                 Tampilkan semua lap + delta\n  export [json|csv]    Cetak laps ke stdout (bisa di-pipe)\n  measure -- <cmd...>  Ukur durasi proses eksternal; exit code diteruskan\n  help                 Bantuan (REPL)\n  exit/quit            Keluar (REPL)\n\nMODES:\n  timer-cli run <cmds...>     # batch (exit code tegas)\n  timer-cli interactive       # REPL eksplisit\n  timer-cli <cmds...>         # kompat-lama (tanpa subcommand)\n  timer-cli                   # REPL default\n"
    );
}
