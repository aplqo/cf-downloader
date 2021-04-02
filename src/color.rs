extern crate termcolor;

use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

macro_rules! get_version {
    ($file:expr) => {
        concat!(
            env!("CARGO_PKG_VERSION"),
            " ",
            include_str!(concat!(env!("OUT_DIR"), "/", $file))
        )
    };
}

pub fn set_fg(stdout: &mut StandardStream, color: Color) {
    stdout
        .set_color(ColorSpec::new().set_fg(Some(color)).set_intense(true))
        .expect("Error: can't set output color");
}
pub fn reset_fg(stdout: &mut StandardStream) {
    stdout
        .set_color(ColorSpec::new().set_fg(None).set_intense(true))
        .expect("Error: Can't reset color");
}

macro_rules! write_color {
    ($dest:expr, $color:expr,$typ:expr,  $($arg:tt)*) => { {
        $crate::color::set_fg($dest, $color);
        write!($dest,"{:>7}: ", $typ);
        $crate::color::reset_fg($dest);
        writeln!($dest, $($arg)*).expect("Failed to write output");
    }
    };
}

macro_rules! write_error {
    ($dest:expr,$typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Red, $typ, $($arg)*);
    };
}

macro_rules! write_info {
    ($dest:expr,$typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Blue, $typ, $($arg)*);
    };
}

macro_rules! write_ok {
    ($dest:expr,$typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Green, $typ, $($arg)*);
    };
}

macro_rules! write_progress {
    ($dest:expr, $typ:expr, $($arg:tt)*) => {
        write_color!($dest, Color::Cyan, $typ, $($arg)*);
    };
}
