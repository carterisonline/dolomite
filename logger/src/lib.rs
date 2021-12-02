use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

use env_logger::fmt::{Color, Style, StyledValue};
use env_logger::Builder;
use log::Level;

pub fn init() {
    try_init().unwrap();
}

pub fn try_init() -> Result<(), log::SetLoggerError> {
    try_init_custom_env("RUST_LOG")
}

pub fn try_init_custom_env(environment_variable_name: &str) -> Result<(), log::SetLoggerError> {
    let mut builder = formatted_builder();

    if let Ok(s) = ::std::env::var(environment_variable_name) {
        builder.parse_filters(&s);
    }

    builder.try_init()
}

pub fn formatted_builder() -> Builder {
    let mut builder = Builder::new();

    builder.format(|f, record| {
        use std::io::Write;
        let target = record.target();
        let max_width = max_target_width(target);

        let mut style = f.style();
        style.set_color(Color::Black);
        let level = colored_level(&mut style, record.level());

        let mut style = f.style();
        let target = style.set_color(Color::Ansi256(245)).value(Padded {
            value: target,
            width: max_width,
        });

        let mut style = f.style();

        let time = style
            .set_bg(Color::White)
            .set_color(Color::Black)
            .value(chrono::Local::now().format(" %S%.6f "));

        writeln!(f, " {}{} {} > {}", time, level, target, record.args(),)
    });

    builder
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_bg(Color::Magenta).value(" TRACE "),
        Level::Debug => style.set_bg(Color::Blue).value(" DEBUG "),
        Level::Info => style.set_bg(Color::Green).value(" INFO "),
        Level::Warn => style.set_bg(Color::Yellow).value(" WARN "),
        Level::Error => style.set_bg(Color::Red).value(" ERROR "),
    }
}
