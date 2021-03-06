use clap::{crate_authors, crate_version, value_t, App, Arg};
use log::debug;
use std::process::Command;

use std::thread::sleep;
use std::time::Duration;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct EndWin;
impl Drop for EndWin {
    fn drop(&mut self) {
        pancurses::endwin();
    }
}

fn main() -> Result<(), failure::Error> {
    env_logger::init();
    debug!("ferris_watch starting...");

    let matches = App::new("ferris_watch")
        .version(crate_version!())
        .author(crate_authors!())
        .about("cute watch command")
        .arg(
            Arg::with_name("command")
                .required(true)
                .multiple(true)
                .help("The command to run periodically"),
        )
        .arg(
            Arg::with_name("interval")
                .long("interval")
                .short("n")
                .takes_value(true)
                .default_value("2.0")
                .help("The period to run a command"),
        )
        .get_matches();
    let command = matches.values_of("command").unwrap().collect::<Vec<_>>();
    let interval = value_t!(matches, "interval", f64)?;
    debug!("command = {:?}", command);
    debug!("interval = {:?}", interval);
    let interval10 = (interval * 10.0) as u32;

    let window = pancurses::initscr();
    let _endwin = EndWin;

    let interruputed = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::SIGINT, interruputed.clone())?;
    'outer: loop {
        let interruputed = || interruputed.load(Ordering::SeqCst);
        let output = Command::new(command[0]).args(&command[1..]).output()?;
        debug!("output = {:?}", output);
        let output = String::from_utf8_lossy(&output.stdout);
        // println!("{}", output);
        window.clear();
        window.printw(output);
        window.refresh();
        for _ in 0..interval10 {
            if interruputed() {
                break 'outer;
            }
            sleep(Duration::from_millis(100));
        }
        if interruputed() {
            break 'outer;
        }
    }
    debug!("end");

    Ok(())
}
