
extern crate yaml_rust;
extern crate reqwest;
extern crate serde_json;
extern crate regex;
extern crate chrono;

#[macro_use]
extern crate lazy_static;

use std::env;
use std::fs;
use std::collections::HashMap;
use yaml_rust::{Yaml,YamlLoader};
use regex::Regex;
use chrono::prelude::*;

static REMIND_TAG: &str = "/remind ";

#[allow(dead_code)]
fn get_config(file: &str) ->
    Result<std::vec::Vec<Yaml>, Box<dyn std::error::Error>>
{
    let s = fs::read_to_string(file)
        .expect("failed to read file");
    let yaml = YamlLoader::load_from_str(&s)?;
    return Ok(yaml);
}

#[allow(dead_code)]
fn pushover(cfg: &yaml_rust::Yaml, msg: &str) ->
    Result<(), Box<dyn std::error::Error>>
{
    let mut map = HashMap::new();
    map.insert("message", msg);
    map.insert("token", cfg["pushover_app_token"].as_str().unwrap());
    map.insert("user", cfg["pushover_user_key"].as_str().unwrap());

    println!("pushover: {} \"{}\"", Local::now(), msg);

    let client = reqwest::Client::new();
    let _resp = client.post("https://api.pushover.net/1/messages.json")
                      .json(&map)
                      .send()?;
    //println!("{:#?}", _resp);

    return Ok(());
}

#[allow(dead_code)]
fn current_weekday(weekday: &str) -> Option<chrono::Weekday>
{
    if weekday.to_lowercase().starts_with("mon") {
        return Some(chrono::Weekday::Mon);
    } else if weekday.to_lowercase().starts_with("tue") {
        return Some(chrono::Weekday::Tue);
    } else if weekday.to_lowercase().starts_with("wed") {
        return Some(chrono::Weekday::Wed);
    } else if weekday.to_lowercase().starts_with("thu") {
        return Some(chrono::Weekday::Thu);
    } else if weekday.to_lowercase().starts_with("fri") {
        return Some(chrono::Weekday::Fri);
    } else if weekday.to_lowercase().starts_with("sat") {
        return Some(chrono::Weekday::Sat);
    } else if weekday.to_lowercase().starts_with("sun") {
        return Some(chrono::Weekday::Sun);
    } else {
        return None;
    }
}

/*
 * Prefixed with "/remind"
 *   <MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM><am|pm>      - Date @ <time>
 *   <MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM>             - Date @ <time> (military)
 *   <MM>/<DD>[/<YY>|/<YYYY>] <HH><am|pm>           - Date @ <time> (min=0)
 *   <MM>/<DD>[/<YY>|/<YYYY>]                       - Date @ 8:00am
 *   <sun|mon|tue|wed|thu|fri|sat> <HH>:<MM><am|pm> - Every <day> of the week @ <time>
 *   <sun|mon|tue|wed|thu|fri|sat> <HH>:<MM>        - Every <day> of the week @ <time> (military)
 *   <sun|mon|tue|wed|thu|fri|sat> <HH><am|pm>      - Every <day> of the week @ <time> (min=0)
 *   <sun|mon|tue|wed|thu|fri|sat>                  - Every <day> of the week @ 8am
 *   monthly                                        - 1st @ 8:00am of every month
 *   biweekly                                       - Every other Monday @ 8:00am (even weeks)
 *   weekly                                         - Every Monday @ 8:00am
 *   daily                                          - Every day @ 8:00am
 *   <HH>:<MM><am|pm>                               - Every day @ <time>
 *   <HH>:<MM>                                      - Every day @ <time> (military)
 *   <HH><am|pm>                                    - Every day @ <time> (min=0)
 */
#[allow(dead_code)]
fn check_reminder(cfg: &yaml_rust::Yaml, r_str: &str) ->
    Result<(), Box<dyn std::error::Error>>
{
    lazy_static! {
        static ref R_DATE_TIME_MIN_AMPM: Regex = Regex::new(r"^/remind ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<year>\d{2}|\d{4}))?) ((?P<hour>\d{1,2}):(?P<min>\d{2})(?P<ampm>am|pm))( (?P<txt>.*))?$").unwrap();
        static ref R_DATE_TIME_MIL: Regex = Regex::new(r"^/remind ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<year>\d{2}|\d{4}))?) ((?P<hour>\d{1,2}):(?P<min>\d{2}))( (?P<txt>.*))?$").unwrap();
        static ref R_DATE_TIME: Regex = Regex::new(r"^/remind ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<year>\d{2}|\d{4}))?) ((?P<hour>\d{1,2})(?P<ampm>am|pm))( (?P<txt>.*))?$").unwrap();
        static ref R_DATE: Regex       = Regex::new(r"^/remind ((?P<month>\d{1,2})/(?P<date>\d{1,2})(/(?P<Y>\d{2}|\d{4}))?)( (?P<txt>.*))?$").unwrap();
        static ref R_DAY_TIME_MIN_AMPM: Regex = Regex::new(r"^/remind (?P<day>sun|mon|tue|wed|thu|fri|sat) ((?P<hour>\d{1,2}):(?P<min>\d{2})(?P<ampm>am|pm))( (?P<txt>.*))?$").unwrap();
        static ref R_DAY_TIME_MIL: Regex = Regex::new(r"^/remind (?P<day>sun|mon|tue|wed|thu|fri|sat) ((?P<hour>\d{1,2}):(?P<min>\d{2}))( (?P<txt>.*))?$").unwrap();
        static ref R_DAY_TIME: Regex   = Regex::new(r"^/remind (?P<day>sun|mon|tue|wed|thu|fri|sat) ((?P<hour>\d{1,2})(?P<ampm>am|pm))( (?P<txt>.*))?$").unwrap();
        static ref R_DAY: Regex        = Regex::new(r"^/remind (?P<day>sun|mon|tue|wed|thu|fri|sat)( (?P<txt>.*))?$").unwrap();
        static ref R_MONTHLY: Regex    = Regex::new(r"^/remind monthly( (?P<txt>.*))?$").unwrap();
        static ref R_BIWEEKLY: Regex   = Regex::new(r"^/remind biweekly( (?P<txt>.*))?$").unwrap();
        static ref R_WEEKLY: Regex     = Regex::new(r"^/remind weekly( (?P<txt>.*))?$").unwrap();
        static ref R_DAILY: Regex      = Regex::new(r"^/remind daily( (?P<txt>.*))?$").unwrap();
        static ref R_DAILY_TIME_MIN_AMPM: Regex = Regex::new(r"^/remind ((?P<hour>\d{1,2}):(?P<min>\d{2})(?P<ampm>am|pm))( (?P<txt>.*))?$").unwrap();
        static ref R_DAILY_TIME_MIL: Regex = Regex::new(r"^/remind ((?P<hour>\d{1,2}):(?P<min>\d{2}))( (?P<txt>.*))?$").unwrap();
        static ref R_DAILY_TIME: Regex = Regex::new(r"^/remind ((?P<hour>\d{1,2})(?P<ampm>am|pm))( (?P<txt>.*))?$").unwrap();

        static ref NOW: DateTime<Local> = Local::now();
    }

    // "<MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM><am|pm>" - Date @ <time>
    if R_DATE_TIME_MIN_AMPM.is_match(r_str) {
        //println!("DATE-TIME-MIN-AMPM -> {}", r_str);
        let c = R_DATE_TIME_MIN_AMPM.captures(r_str).unwrap();

        let mut year = NOW.year();
        if c.name("year") != None {
            year = c.name("year").unwrap().as_str().parse::<i32>().unwrap();
            if year <= 99 {
                year = year + 2000;
            }
        }

        let mut hour = c.name("hour").unwrap().as_str().parse::<u32>().unwrap();
        if (c.name("ampm").unwrap().as_str() == "pm") && (hour < 12) {
            hour = hour + 12;
        }

        if (NOW.month() ==
            c.name("month").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.day() ==
            c.name("date").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.year() == year) &&
           (NOW.hour() == hour) &&
           (NOW.minute() ==
            c.name("min").unwrap().as_str().parse::<u32>().unwrap()) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<MM>/<DD>[/<YY>|/<YYYY>] <HH>:<MM>" - Date @ <time>
    else if R_DATE_TIME_MIL.is_match(r_str) {
        //println!("DATE-TIME-MIL -> {}", r_str);
        let c = R_DATE_TIME_MIL.captures(r_str).unwrap();

        let mut year = NOW.year();
        if c.name("year") != None {
            year = c.name("year").unwrap().as_str().parse::<i32>().unwrap();
            if year <= 99 {
                year = year + 2000;
            }
        }

        if (NOW.month() ==
            c.name("month").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.day() ==
            c.name("date").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.year() == year) &&
           (NOW.hour() ==
            c.name("hour").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.minute() ==
            c.name("min").unwrap().as_str().parse::<u32>().unwrap()) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<MM>/<DD>[/<YY>|/<YYYY>] <HH><am|pm>" - Date @ <time>
    else if R_DATE_TIME.is_match(r_str) {
        //println!("DATE-TIME -> {}", r_str);
        let c = R_DATE_TIME.captures(r_str).unwrap();

        let mut year = NOW.year();
        if c.name("year") != None {
            year = c.name("year").unwrap().as_str().parse::<i32>().unwrap();
            if year <= 99 {
                year = year + 2000;
            }
        }

        let mut hour = c.name("hour").unwrap().as_str().parse::<u32>().unwrap();
        if (c.name("ampm").unwrap().as_str() == "pm") && (hour < 12) {
            hour = hour + 12;
        }

        if (NOW.month() ==
            c.name("month").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.day() ==
            c.name("date").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.year() == year) &&
           (NOW.hour() == hour) &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<MM>/<DD>[/<YY>|/<YYYY>]" - Date @ 8am
    else if R_DATE.is_match(r_str) {
        //println!("DATE -> {}", r_str);
        let c = R_DATE.captures(r_str).unwrap();

        let mut year = NOW.year();
        if c.name("year") != None {
            year = c.name("year").unwrap().as_str().parse::<i32>().unwrap();
            if year <= 99 {
                year = year + 2000;
            }
        }

        if (NOW.month() ==
            c.name("month").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.day() ==
            c.name("date").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.year() == year) &&
           (NOW.hour() == 8)  &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<sun|mon|tue|wed|thu|fri|sat> <HH>:<MM><am|pm>" - Every <day> of the week @ <time>
    else if R_DAY_TIME_MIN_AMPM.is_match(r_str) {
        //println!("DAY-TIME-MIN-AMPM -> {}", r_str);
        let c = R_DAY_TIME_MIN_AMPM.captures(r_str).unwrap();

        let dow = current_weekday(c.name("day").unwrap().as_str());

        let mut hour = c.name("hour").unwrap().as_str().parse::<u32>().unwrap();
        if (c.name("ampm").unwrap().as_str() == "pm") && (hour < 12) {
            hour = hour + 12;
        }

        if (dow != None) &&
           (NOW.weekday() == dow.unwrap()) &&
           (NOW.hour() == hour) &&
           (NOW.minute() ==
            c.name("min").unwrap().as_str().parse::<u32>().unwrap()) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<sun|mon|tue|wed|thu|fri|sat> <HH>:<MM>" - Every <day> of the week @ <time>
    else if R_DAY_TIME_MIL.is_match(r_str) {
        //println!("DAY-TIME-MIL -> {}", r_str);
        let c = R_DAY_TIME_MIL.captures(r_str).unwrap();

        let dow = current_weekday(c.name("day").unwrap().as_str());

        if (dow != None) &&
           (NOW.weekday() == dow.unwrap()) &&
           (NOW.hour() ==
            c.name("hour").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.minute() ==
            c.name("min").unwrap().as_str().parse::<u32>().unwrap()) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<sun|mon|tue|wed|thu|fri|sat> <HH><am|pm>" - Every <day> of the week @ <time>
    else if R_DAY_TIME.is_match(r_str) {
        //println!("DAY-TIME -> {}", r_str);
        let c = R_DAY_TIME.captures(r_str).unwrap();

        let dow = current_weekday(c.name("day").unwrap().as_str());

        let mut hour = c.name("hour").unwrap().as_str().parse::<u32>().unwrap();
        if (c.name("ampm").unwrap().as_str() == "pm") && (hour < 12) {
            hour = hour + 12;
        }

        if (dow != None) &&
           (NOW.weekday() == dow.unwrap()) &&
           (NOW.hour() == hour) &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<sun|mon|tue|wed|thu|fri|sat>" - Every <day> of the week @ 8am
    else if R_DAY.is_match(r_str) {
        //println!("DAY -> {}", r_str);
        let c = R_DAY.captures(r_str).unwrap();

        let dow = current_weekday(c.name("day").unwrap().as_str());

        if (dow != None) &&
           (NOW.weekday() == dow.unwrap()) &&
           (NOW.hour() == 8)  &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "/remind monthly" - 1st @ 8am of every month
    else if R_MONTHLY.is_match(r_str) {
        //println!("MONTHLY -> {}", r_str);
        let c = R_MONTHLY.captures(r_str).unwrap();

        if (NOW.day() == 1) &&
           (NOW.hour() == 8) &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "/remind biweekly" - Every other Monday @ 8am (even weeks)
    else if R_BIWEEKLY.is_match(r_str) {
        //println!("BIWEEKLY -> {}", r_str);
        let c = R_BIWEEKLY.captures(r_str).unwrap();

        if (NOW.weekday() == chrono::Weekday::Mon) &&
           ((NOW.iso_week().week() % 2) == 0) &&
           (NOW.hour() == 8) &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "/remind weekly" - Every Monday @ 8am
    else if R_WEEKLY.is_match(r_str) {
        //println!("WEEKLY -> {}", r_str);
        let c = R_WEEKLY.captures(r_str).unwrap();

        if (NOW.weekday() == chrono::Weekday::Mon) &&
           (NOW.hour() == 8) &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "/remind daily" - Every day @ 8am
    else if R_DAILY.is_match(r_str) {
        //println!("DAILY -> {}", r_str);
        let c = R_DAILY.captures(r_str).unwrap();

        if (NOW.hour() == 8) &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<HH>:<MM><am|pm>" - Every day @ <time>
    else if R_DAILY_TIME_MIN_AMPM.is_match(r_str) {
        //println!("DAILY-TIME-MIN-AMPM -> {}", r_str);
        let c = R_DAILY_TIME_MIN_AMPM.captures(r_str).unwrap();

        let mut hour = c.name("hour").unwrap().as_str().parse::<u32>().unwrap();
        if (c.name("ampm").unwrap().as_str() == "pm") && (hour < 12) {
            hour = hour + 12;
        }

        if (NOW.hour() == hour) &&
           (NOW.minute() ==
            c.name("min").unwrap().as_str().parse::<u32>().unwrap()) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<HH>:<MM>" - Every day @ <time>
    else if R_DAILY_TIME_MIL.is_match(r_str) {
        //println!("DAILY-TIME-MIL -> {}", r_str);
        let c = R_DAILY_TIME_MIL.captures(r_str).unwrap();

        if (NOW.hour() ==
            c.name("hour").unwrap().as_str().parse::<u32>().unwrap()) &&
           (NOW.minute() ==
            c.name("min").unwrap().as_str().parse::<u32>().unwrap()) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }
    // "<HH><am|pm>" - Every day @ <time>
    else if R_DAILY_TIME.is_match(r_str) {
        //println!("DAILY-TIME -> {}", r_str);
        let c = R_DAILY_TIME.captures(r_str).unwrap();

        let mut hour = c.name("hour").unwrap().as_str().parse::<u32>().unwrap();
        if (c.name("ampm").unwrap().as_str() == "pm") && (hour < 12) {
            hour = hour + 12;
        }

        if (NOW.hour() == hour) &&
           (NOW.minute() == 0) {
            pushover(cfg, c.name("txt").unwrap().as_str())?;
        }
    }

    return Ok(());
}

#[allow(dead_code)]
fn test_reminders(cfg: &yaml_rust::Yaml) ->
    Result<(), Box<dyn std::error::Error>>
{
    let r_tests = "x/remind monthly test
                   x/remind weekly test
                   x/remind daily test
                   x/remind biweekly test
                   x/remind sun test
                   x/remind sun 7am test
                   x/remind sun 7pm test
                   x/remind sun 7pm test
                   x/remind sun 10am test
                   x/remind sun 10pm test
                   x/remind 3/17 test
                   x/remind 3/17 7am test
                   x/remind 3/17 7pm test
                   x/remind 3/17 10am test
                   x/remind 3/17 10pm test
                   x/remind 3/17/19 test
                   x/remind 3/17/19 7am test
                   x/remind 3/17/19 7pm test
                   x/remind 3/17/19 10am test
                   x/remind 3/17/19 10pm test
                   x/remind 3/17/2019 test
                   /remind 4/29/2020 1pm test3
                   /remind 4/29/2020 1:00pm test3a
                   /remind 4/29/2020 13:00 test3m
                   /remind 4/29/20 1pm test4
                   /remind 4/29/20 1:00pm test4a
                   /remind 4/29/20 13:00 test4m
                   /remind 4/29 1pm test5
                   /remind 4/29 1:00pm test5a
                   /remind 4/29 13:00 test5m
                   /remind wed 1pm test6
                   /remind wed 1:00pm test6a
                   /remind wed 13:00 test6m
                   /remind 1pm test7
                   /remind 1:00pm test7a
                   /remind 13:00 test7m
                   x/remind 3/17/2019 7am test
                   x/remind 3/17/2019 7pm test
                   x/remind 3/17/2019 10am test
                   x/remind 3/17/2019 10pm test";

    r_tests.lines().for_each(|line| {
        let l = line.trim();
        if l.starts_with(REMIND_TAG) {
            println!("testing: {:?}", l);
            let _rc = check_reminder(cfg, l);
        }
    });

    return Ok(());
}

#[allow(dead_code)]
fn get_todo(cfg: &yaml_rust::Yaml) ->
    Result<String, Box<dyn std::error::Error>>
{
    /* XXX
     * check http_auth config
     * test file_url with file:// for a local file
     */
    let client = reqwest::Client::new();
    let txt = client.get(cfg["file_url"].as_str().unwrap())
                    .basic_auth(cfg["http_username"].as_str().unwrap(),
                                Some(cfg["http_password"].as_str().unwrap()))
                    .send()?
                    .text()?;
    return Ok(txt);
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("ERROR: invalid command line args!");
        println!("Usage: {} <config.yaml>", args[0]);
        return Ok(());
    }

    let cfg = &get_config(&args[1])?[0]; // select the first document
    //println!("{:?}", cfg);

    //pushover(cfg, "Test from Rust!")?;

    //test_reminders(cfg)?;

    let txt = get_todo(cfg)?;
    txt.lines().for_each(|line| {
        //println!("{}", line);
        if line.starts_with(REMIND_TAG) {
            //println!("{:?}", line);
            let _rc = check_reminder(cfg, line);
        }
    });

    return Ok(());
}

