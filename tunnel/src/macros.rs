
// @TODO: replace with proper error handling/reporting.
macro_rules! println_err {
    ($fmt:literal $(, $arg:expr)*) => {
        println!(concat!("ERROR ({}:{}): ", $fmt), file!(), line!() $(, $arg)*)
    };
}

macro_rules! println_dbg {
    ($fmt:literal $(, $arg:expr)*) => {
        println!(concat!("DEBUG ({}:{}): ", $fmt), file!(), line!() $(, $arg)*)
    };
}
