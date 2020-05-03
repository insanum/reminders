
extern crate yaml_rust;
extern crate reqwest;
extern crate regex;
extern crate chrono;
extern crate getopts;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs;
use std::collections::HashMap;
use getopts::Options;
use yaml_rust::{Yaml,YamlLoader};
use regex::{Regex,Match};
use chrono::prelude::*;
use chrono::{NaiveDate,NaiveTime,NaiveDateTime};

static REMIND_TAG: &str = "/remind ";

/* Read in the YAML config file and parse it. */
fn get_config(file: &str) ->
    Result<std::vec::Vec<Yaml>, Box<dyn std::error::Error>>
{
    let s = fs::read_to_string(file)
                .expect("failed to read config file");
    let yaml = YamlLoader::load_from_str(&s)?;
    return Ok(yaml);
}

/* Send the message (i.e. reminder text) to pushover. */
fn pushover(cfg: &yaml_rust::Yaml, dt: NaiveDateTime, msg: &str) ->
    Result<(), Box<dyn std::error::Error>>
{
    let app_token = &cfg["pushover_app_token"];
    let user_key = &cfg["pushover_user_key"];

    if app_token.is_badvalue() || app_token.is_null() ||
       user_key.is_badvalue() || user_key.is_null() {
        println!("reminder: {:?} \"{}\"", dt, msg);
        return Ok(());
    }

    let mut map = HashMap::new();
    map.insert("message", msg);
    map.insert("token", app_token.as_str().unwrap());
    map.insert("user", user_key.as_str().unwrap());

    println!("pushover: {:?} \"{}\"", dt, msg);

    let client = reqwest::Client::new();
    let _resp = client.post("https://api.pushover.net/1/messages.json")
                      .json(&map)
                      .send()?;
    //println!("{:#?}", _resp);

    return Ok(());
}

/* All reminder strings are prefixed with "/remind ..." */
lazy_static!
{
    /* <MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM><am|pm> - Date @ <time> */
    static ref R_DATE_TIME_MIN_AMPM: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<year>\d{2}|\d{4}))?)
        \s+
        ((?P<hour>\d{1,2}):(?P<min>\d{2})(?P<ampm>am|pm))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM> - Date @ <time> (military) */
    static ref R_DATE_TIME_MIL: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<year>\d{2}|\d{4}))?)
        \s+
        ((?P<hour>\d{1,2}):(?P<min>\d{2}))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <MM>/<DD>[/<YY>|/<YYYY>] <HH><am|pm> - Date @ <time> (min=0) */
    static ref R_DATE_TIME: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<year>\d{2}|\d{4}))?)
        \s+
        ((?P<hour>\d{1,2})(?P<ampm>am|pm))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <MM>/<DD>[/<YY>|/<YYYY>] - Date @ 8:00am */
    static ref R_DATE: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<Y>\d{2}|\d{4}))?)
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <sun|mon|tue|wed|thu|fri|sat> <HH>:<MM><am|pm> - Every <day> of the week @ <time> */
    static ref R_DAY_TIME_MIN_AMPM: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        (?P<day>sun|mon|tue|wed|thu|fri|sat)
        \s+
        ((?P<hour>\d{1,2}):(?P<min>\d{2})(?P<ampm>am|pm))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <sun|mon|tue|wed|thu|fri|sat> <HH>:<MM> - Every <day> of the week @ <time> (military) */
    static ref R_DAY_TIME_MIL: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        (?P<day>sun|mon|tue|wed|thu|fri|sat)
        \s+
        ((?P<hour>\d{1,2}):(?P<min>\d{2}))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <sun|mon|tue|wed|thu|fri|sat> <HH><am|pm> - Every <day> of the week @ <time> (min=0) */
    static ref R_DAY_TIME: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        (?P<day>sun|mon|tue|wed|thu|fri|sat)
        \s+
        ((?P<hour>\d{1,2})(?P<ampm>am|pm))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <sun|mon|tue|wed|thu|fri|sat> - Every <day> of the week @ 8am */
    static ref R_DAY: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        (?P<day>sun|mon|tue|wed|thu|fri|sat)
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* monthly - On the 1st @ 8:00am of every month */
    static ref R_MONTHLY: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        monthly
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* biweekly - Every other Monday @ 8:00am (even weeks) */
    static ref R_BIWEEKLY: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        biweekly
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* weekly - Every Monday @ 8:00am */
    static ref R_WEEKLY: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        weekly
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* daily - Every day @ 8:00am */
    static ref R_DAILY: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        daily
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <HH>:<MM><am|pm> - Every day @ <time> */
    static ref R_DAILY_TIME_MIN_AMPM: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        ((?P<hour>\d{1,2}):(?P<min>\d{2})(?P<ampm>am|pm))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <HH>:<MM> - Every day @ <time> (military) */
    static ref R_DAILY_TIME_MIL: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        ((?P<hour>\d{1,2}):(?P<min>\d{2}))
        \s+
        (?P<txt>.*)$
        ").unwrap();

    /* <HH><am|pm> - Every day @ <time> (min=0) */
    static ref R_DAILY_TIME: Regex = Regex::new(r"(?x)
        ^/remind
        \s+
        ((?P<hour>\d{1,2})(?P<ampm>am|pm))
        \s+
        (?P<txt>.*)$
        ").unwrap();
}

fn get_weekday(weekday: Option<Match>) -> Option<chrono::Weekday>
{
    if weekday == None {
        return None;
    }

    let wd = weekday.unwrap().as_str();

    if wd.to_lowercase().starts_with("mon") {
        return Some(chrono::Weekday::Mon);
    } else if wd.to_lowercase().starts_with("tue") {
        return Some(chrono::Weekday::Tue);
    } else if wd.to_lowercase().starts_with("wed") {
        return Some(chrono::Weekday::Wed);
    } else if wd.to_lowercase().starts_with("thu") {
        return Some(chrono::Weekday::Thu);
    } else if wd.to_lowercase().starts_with("fri") {
        return Some(chrono::Weekday::Fri);
    } else if wd.to_lowercase().starts_with("sat") {
        return Some(chrono::Weekday::Sat);
    } else if wd.to_lowercase().starts_with("sun") {
        return Some(chrono::Weekday::Sun);
    } else {
        return None;
    }
}

fn get_year(dt: NaiveDateTime, year: Option<Match>) -> i32
{
    let mut y = dt.year();

    if year != None {
        y = year.unwrap().as_str().parse::<i32>().unwrap();
        if y <= 99 {
            y = y + 2000;
        }
    }

    return y;
}

fn get_hour(hour: Option<Match>, ampm: Option<Match>) -> u32
{
    let mut h = hour.unwrap().as_str().parse::<u32>().unwrap();
    if (ampm.unwrap().as_str() == "pm") && (h < 12) {
        h = h + 12;
    }

    return h;
}

/*
 * Match the reminder line against all teh regex's. If there is a match
 * against the current time then send a notification message via pushover.
 */
fn check_reminder(cfg: &yaml_rust::Yaml, dt: NaiveDateTime, r_str: &str) ->
    Result<(), Box<dyn std::error::Error>>
{
    /* "<MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM><am|pm>" - Date @ <time> */
    if R_DATE_TIME_MIN_AMPM.is_match(r_str) {
        //println!("DATE-TIME-MIN-AMPM -> {}", r_str);
        let c = R_DATE_TIME_MIN_AMPM.captures(r_str).unwrap();

        let year = get_year(dt, c.name("year"));
        let hour = get_hour(c.name("hour"), c.name("ampm"));

        let d = NaiveDateTime::new(
                    NaiveDate::from_ymd(
                        year,
                        c.name("month").unwrap().as_str().parse::<u32>()?,
                        c.name("date").unwrap().as_str().parse::<u32>()?),
                    NaiveTime::from_hms(
                        hour,
                        c.name("min").unwrap().as_str().parse::<u32>()?,
                        0));

        if dt == d {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM>" - Date @ <time> */
    else if R_DATE_TIME_MIL.is_match(r_str) {
        //println!("DATE-TIME-MIL -> {}", r_str);
        let c = R_DATE_TIME_MIL.captures(r_str).unwrap();

        let year = get_year(dt, c.name("year"));

        let d = NaiveDateTime::new(
                    NaiveDate::from_ymd(
                        year,
                        c.name("month").unwrap().as_str().parse::<u32>()?,
                        c.name("date").unwrap().as_str().parse::<u32>()?),
                    NaiveTime::from_hms(
                        c.name("hour").unwrap().as_str().parse::<u32>()?,
                        c.name("min").unwrap().as_str().parse::<u32>()?,
                        0));

        if dt == d {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<MM>/<DD>[/<YY>|/<YYYY>] <HH><am|pm>" - Date @ <time> */
    else if R_DATE_TIME.is_match(r_str) {
        //println!("DATE-TIME -> {}", r_str);
        let c = R_DATE_TIME.captures(r_str).unwrap();

        let year = get_year(dt, c.name("year"));
        let hour = get_hour(c.name("hour"), c.name("ampm"));

        let d = NaiveDateTime::new(
                    NaiveDate::from_ymd(
                        year,
                        c.name("month").unwrap().as_str().parse::<u32>()?,
                        c.name("date").unwrap().as_str().parse::<u32>()?),
                    NaiveTime::from_hms(
                        hour,
                        0,
                        0));

        if dt == d {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<MM>/<DD>[/<YY>|/<YYYY>]" - Date @ 8am */
    else if R_DATE.is_match(r_str) {
        //println!("DATE -> {}", r_str);
        let c = R_DATE.captures(r_str).unwrap();

        let year = get_year(dt, c.name("year"));

        let d = NaiveDateTime::new(
                    NaiveDate::from_ymd(
                        year,
                        c.name("month").unwrap().as_str().parse::<u32>()?,
                        c.name("date").unwrap().as_str().parse::<u32>()?),
                    NaiveTime::from_hms(
                        8,
                        0,
                        0));

        if dt == d {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<sun|mon|tue|wed|thu|fri|sat> <HH>:<MM><am|pm>" - Every <day> of the week @ <time> */
    else if R_DAY_TIME_MIN_AMPM.is_match(r_str) {
        //println!("DAY-TIME-MIN-AMPM -> {}", r_str);
        let c = R_DAY_TIME_MIN_AMPM.captures(r_str).unwrap();

        let dow = get_weekday(c.name("day"));
        let hour = get_hour(c.name("hour"), c.name("ampm"));

        let t = NaiveTime::from_hms(
                    hour,
                    c.name("min").unwrap().as_str().parse::<u32>()?,
                    0);

        if (dow != None) &&
           (dt.weekday() == dow.unwrap()) &&
           (dt.time() == t) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<sun|mon|tue|wed|thu|fri|sat> <HH>:<MM>" - Every <day> of the week @ <time> */
    else if R_DAY_TIME_MIL.is_match(r_str) {
        //println!("DAY-TIME-MIL -> {}", r_str);
        let c = R_DAY_TIME_MIL.captures(r_str).unwrap();

        let dow = get_weekday(c.name("day"));

        let t = NaiveTime::from_hms(
                    c.name("hour").unwrap().as_str().parse::<u32>()?,
                    c.name("min").unwrap().as_str().parse::<u32>()?,
                    0);

        if (dow != None) &&
           (dt.weekday() == dow.unwrap()) &&
           (dt.time() == t) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<sun|mon|tue|wed|thu|fri|sat> <HH><am|pm>" - Every <day> of the week @ <time> */
    else if R_DAY_TIME.is_match(r_str) {
        //println!("DAY-TIME -> {}", r_str);
        let c = R_DAY_TIME.captures(r_str).unwrap();

        let dow = get_weekday(c.name("day"));
        let hour = get_hour(c.name("hour"), c.name("ampm"));

        let t = NaiveTime::from_hms(
                    hour,
                    0,
                    0);

        if (dow != None) &&
           (dt.weekday() == dow.unwrap()) &&
           (dt.time() == t) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<sun|mon|tue|wed|thu|fri|sat>" - Every <day> of the week @ 8am */
    else if R_DAY.is_match(r_str) {
        //println!("DAY -> {}", r_str);
        let c = R_DAY.captures(r_str).unwrap();

        let dow = get_weekday(c.name("day"));

        if (dow != None) &&
           (dt.weekday() == dow.unwrap()) &&
           (dt.hour() == 8)  &&
           (dt.minute() == 0) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "/remind monthly" - 1st @ 8am of every month */
    else if R_MONTHLY.is_match(r_str) {
        //println!("MONTHLY -> {}", r_str);
        let c = R_MONTHLY.captures(r_str).unwrap();

        if (dt.day() == 1) &&
           (dt.hour() == 8) &&
           (dt.minute() == 0) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "/remind biweekly" - Every other Monday @ 8am (even weeks) */
    else if R_BIWEEKLY.is_match(r_str) {
        //println!("BIWEEKLY -> {}", r_str);
        let c = R_BIWEEKLY.captures(r_str).unwrap();

        if (dt.weekday() == chrono::Weekday::Mon) &&
           ((dt.iso_week().week() % 2) == 0) &&
           (dt.hour() == 8) &&
           (dt.minute() == 0) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "/remind weekly" - Every Monday @ 8am */
    else if R_WEEKLY.is_match(r_str) {
        //println!("WEEKLY -> {}", r_str);
        let c = R_WEEKLY.captures(r_str).unwrap();

        if (dt.weekday() == chrono::Weekday::Mon) &&
           (dt.hour() == 8) &&
           (dt.minute() == 0) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "/remind daily" - Every day @ 8am */
    else if R_DAILY.is_match(r_str) {
        //println!("DAILY -> {}", r_str);
        let c = R_DAILY.captures(r_str).unwrap();

        if (dt.hour() == 8) &&
           (dt.minute() == 0) {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<HH>:<MM><am|pm>" - Every day @ <time> */
    else if R_DAILY_TIME_MIN_AMPM.is_match(r_str) {
        //println!("DAILY-TIME-MIN-AMPM -> {}", r_str);
        let c = R_DAILY_TIME_MIN_AMPM.captures(r_str).unwrap();

        let hour = get_hour(c.name("hour"), c.name("ampm"));

        let t = NaiveTime::from_hms(
                    hour,
                    c.name("min").unwrap().as_str().parse::<u32>()?,
                    0);

        if dt.time() == t {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<HH>:<MM>" - Every day @ <time> */
    else if R_DAILY_TIME_MIL.is_match(r_str) {
        //println!("DAILY-TIME-MIL -> {}", r_str);
        let c = R_DAILY_TIME_MIL.captures(r_str).unwrap();

        let t = NaiveTime::from_hms(
                    c.name("hour").unwrap().as_str().parse::<u32>()?,
                    c.name("min").unwrap().as_str().parse::<u32>()?,
                    0);

        if dt.time() == t {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }
    /* "<HH><am|pm>" - Every day @ <time> */
    else if R_DAILY_TIME.is_match(r_str) {
        //println!("DAILY-TIME -> {}", r_str);
        let c = R_DAILY_TIME.captures(r_str).unwrap();

        let hour = get_hour(c.name("hour"), c.name("ampm"));

        let t = NaiveTime::from_hms(
                    hour,
                    0,
                    0);

        if dt.time() == t {
            pushover(cfg, dt, c.name("txt").unwrap().as_str())?;
        }
    }

    return Ok(());
}

/*
 * Get the specified file that contains the reminder strings. This function
 * will fetch the file over HTTP (w/ basic auth if specified) or read the
 * file directory from local disk.
 */
fn get_todo(cfg: &yaml_rust::Yaml) ->
    Result<String, Box<dyn std::error::Error>>
{
    let file = &cfg["file"];
    let txt;

    if file.is_badvalue() || file.is_null() {
        if cfg["reminders"].is_badvalue() {
            return Err("ERROR: invalid config file")?;
        } else {
            return Ok("".to_string());
        }
    }

    if file.as_str().unwrap().starts_with("http") {
        let client = reqwest::Client::new();

        let auth = &cfg["http_auth"];

        if !auth.is_badvalue() && (auth.as_str().unwrap() == "basic") {
            let user = &cfg["http_username"];
            let pass = &cfg["http_password"];

            if user.is_badvalue() || user.is_null() ||
               pass.is_badvalue() || pass.is_null() {
                return Err("ERROR: invalid http credentials")?;
            }

            txt = client.get(file.as_str().unwrap())
                        .basic_auth(user.as_str().unwrap(),
                                    Some(pass.as_str().unwrap()))
                        .send()?
                        .text()?;
        } else {
            txt = client.get(file.as_str().unwrap())
                        .send()?
                        .text()?;
        }
    } else {
        txt = fs::read_to_string(file.as_str().unwrap())
                  .expect("failed to read reminder file");
    }

    //println!("{:?}", txt);
    return Ok(txt);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "", "config file", "<file.yaml>");
    opts.optopt("t", "", "time override '<YYYY/MM/DD HH:MM>'", "<timestamp>");
    opts.optflag("p", "pushover", "send test message to pushover");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => return Err(f.to_string())?
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    let dt: NaiveDateTime;
    if matches.opt_present("t") {
        dt = NaiveDateTime::parse_from_str(&matches.opt_str("t").unwrap(),
                                           "%Y/%m/%d %H:%M")?;
    } else {
        dt = Local::now().naive_local().with_nanosecond(0).unwrap()
                                       .with_second(0).unwrap();
    }
    println!("@ {:?}", dt);

    if !matches.opt_present("c") {
        return Err("ERROR: must specify the config file")?;
    }

    let cfg_file = matches.opt_str("c").unwrap();

    let cfg = &get_config(&cfg_file)?[0]; /* select the first document */
    //println!("{:?}", cfg);

    if matches.opt_present("p") {
        pushover(cfg, dt, "Test from Rust::reminders!")?;
        return Ok(());
    }

    let rtxt = &cfg["reminders"];
    if !rtxt.is_badvalue() && !rtxt.is_null() {
        rtxt.as_str().unwrap().lines().for_each(|line| {
            if line.starts_with(REMIND_TAG) {
                //println!("{:?}", line);
                let _rc = check_reminder(cfg, dt, line);
            }
        });
    }

    let txt = get_todo(cfg)?;
    txt.lines().for_each(|line| {
        if line.starts_with(REMIND_TAG) {
            //println!("{:?}", line);
            let _rc = check_reminder(cfg, dt, line);
        }
    });

    return Ok(());
}

