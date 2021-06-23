use chrono::{DateTime, Local, TimeZone};
use clap::{App, Arg};
use std::mem::zeroed;

struct Clock {}

impl Clock {
    /// cargo run -q -- get -s rfc2822
    pub fn get() -> DateTime<Local> {
        Local::now()
    }
    /// takes in datetime in specified timezone
    /// cargo run -q -- set -s rfc2822 "Wed, 23 Jun 2021 18:47:37 +0300"
    #[cfg(not(windows))]
    pub fn set<Tz: TimeZone>(t: DateTime<Tz>) -> () {
        // archaic shit for working with time from C's standard library
        use libc::settimeofday;
        // first two are type aliases, timeval is a struct
        // time_t = seconds that have elapsed since epoch
        // suseconds_t = fractional component of current second
        // both are used to construct the timeval struct
        use libc::{suseconds_t, time_t, timeval, timezone};

        // instantiate time in local timezone
        let t = t.with_timezone(&Local);

        // construct the timeval struct
        let mut u: timeval = unsafe { zeroed() };
        u.tv_sec = t.timestamp() as time_t;
        u.tv_usec = t.timestamp_subsec_micros() as suseconds_t;

        //todo faking it..
        println!("setting internal clock to {} epoch with {} fractional component", u.tv_sec, u.tv_usec);
        // unsafe {
        //     // has to be set to null or won't work - archaic C...
        //     // needs to be cast as a raw pointer
        //     let mock_tz: *const timezone = std::ptr::null();
        //     // takes 2 args: time value and timezone
        //     // needs to be cast as a raw pointer
        //     settimeofday(&u as *const timeval, mock_tz);
        // }
    }

    #[cfg(windows)]
    pub fn set() -> () {
        todo!()
    }
}

fn main() {
    let app = App::new("clock")
        .version("0.1")
        .about("simple clock app")
        .arg(Arg::with_name("action")
            .takes_value(true)
            .possible_values(&["get", "set"])
            .default_value("get"))
        .arg(Arg::with_name("std")
            .short("s")
            .long("use-standard")
            .takes_value(true)
            .possible_values(&["rfc2822", "rfc3339", "timestamp"])
            .default_value("rfc3339"))
        .arg(Arg::with_name("datetime"))
        .help("when <action> is 'set' apply <datetime> else ignore");

    let args = app.get_matches();

    let action = args.value_of("action").unwrap();
    let std = args.value_of("std").unwrap();

    if action == "set" {
        let t_ = args.value_of("datetime").unwrap();

        let parser = match std {
            "rfc2822" => DateTime::parse_from_rfc2822,
            "rfc3339" => DateTime::parse_from_rfc3339,
            _ => todo!(),
        };

        let err_msg = format!("unable to parse time");
        let t = parser(t_).expect(&err_msg);

        Clock::set(t);

        //workaround to have errors but not have to return a result from this function
        let maybe_error = std::io::Error::last_os_error();
        let os_error_code = &maybe_error.raw_os_error();
        match os_error_code {
            Some(0) => (),
            None => (),
            _ => eprintln!("unable to set the time, {:?}", maybe_error),
        }
    }

    let now = Clock::get();
    match std {
        "rfc2822" => println!("{}", now.to_rfc2822()),
        "rfc3339" => println!("{}", now.to_rfc3339()),
        "timestamp" => println!("{}", now.timestamp()),
        _ => unreachable!()
    }
}
