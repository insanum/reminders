#[macro_use]
extern crate lazy_static;

use colored::*;
use getopts::Options;
use regex::Regex;
use std::env;
use std::fs;

lazy_static! {
    static ref TASK_LINE: Regex = Regex::new(r"^(\s*)(-\s\[[ |x]\]\s)(.*)$").unwrap();
    static ref TASK_NO_X_LINE: Regex = Regex::new(r"^(\s*)(-\s\[ \]\s)(.*)$").unwrap();
    static ref TASK_X_LINE: Regex = Regex::new(r"^(\s*)(-\s\[x\]\s)(.*)$").unwrap();
    static ref HDR_LINE: Regex = Regex::new(r"^(#+)\s(.*)").unwrap();
    static ref TAG: Regex = Regex::new(r"^#\S+$").unwrap();
    static ref BLANK: Regex = Regex::new(r"^\s*$").unwrap();
}

fn tag_color(tag: &str) -> String {
    match tag {
        "#high" => tag.red().to_string(),
        "#medium" => tag.yellow().to_string(),
        "#low" => tag.green().to_string(),
        _ => tag.magenta().to_string(),
    }
}

fn dump_line(i: usize, line: &str) {
    let caps = match TASK_LINE.captures(line) {
        None => return,
        Some(c) => c,
    };

    let x = caps
        .get(2)
        .map(|m| match m.as_str() {
            "- [x] " => true,
            _ => false,
        })
        .unwrap();

    let words: Vec<&str> = caps.get(3).map_or("", |m| m.as_str()).split(' ').collect();

    let mut lp = String::new();

    for i in 0..words.len() {
        if TAG.is_match(&words[i]) {
            lp.push_str(&tag_color(&words[i]));
        } else {
            lp.push_str(words[i]);
        }

        if i != (words.len() - 1) {
            lp.push_str(" ");
        }
    }

    println!(
        "{}: {}{}",
        (i + 1).to_string().cyan(),
        caps.get(1).unwrap().as_str(),
        match x {
            true => lp.as_str().dimmed().italic().strikethrough(),
            false => lp.as_str().clear(),
        }
    );
}

fn dump_lines(lines: &Vec<&str>, sec_start: i32, sec_end: i32, show_completed: bool) {
    let mut indent = 0;

    for i in sec_start..(sec_end + 1) {
        let idx = i as usize;
        if BLANK.is_match(lines[idx]) {
            continue;
        } else if HDR_LINE.is_match(lines[idx]) {
            let (hl, ht) = lines[idx].split_once(' ').unwrap();
            let mut hp = String::new();
            for _j in 0..(hl.len() - 1) {
                hp.push_str("  ");
            }
            println!("{}", format!("{}{}:", hp, ht.bold().underline()));
            indent = hl.len();
        } else if TASK_NO_X_LINE.is_match(lines[idx])
            || (show_completed && TASK_X_LINE.is_match(lines[idx]))
        {
            let mut hp = String::new();
            for _j in 0..indent {
                hp.push_str("  ");
            }
            print!("{}", hp);
            dump_line(idx, lines[idx]);
        }
    }
}

fn dump_tag(lines: &Vec<&str>, sec_start: i32, sec_end: i32, tag: String, show_completed: bool) {
    let tag_match = Regex::new(&format!(
        r"^\s*- \[[{}]\] .*\s+#(?i){}(?-i)($|\s+.*$)",
        match show_completed {
            true => " |x",
            false => " ",
        },
        tag
    ))
    .unwrap();
    let mut parent = false;
    let mut indent = 0;

    for i in sec_start..(sec_end + 1) {
        let idx = i as usize;
        if BLANK.is_match(lines[idx]) {
            continue;
        } else if HDR_LINE.is_match(lines[idx]) {
            let (hl, ht) = lines[idx].split_once(' ').unwrap();
            let mut hp = String::new();
            for _j in 0..(hl.len() - 1) {
                hp.push_str("  ");
            }
            println!("{}", format!("{}{}:", hp, ht.bold().underline()));
            indent = hl.len();
        }

        if parent {
            if TASK_LINE.is_match(lines[idx]) && lines[idx].starts_with(" ") {
                let mut hp = String::new();
                for _j in 0..indent {
                    hp.push_str("  ");
                }
                print!("{}", hp);
                dump_line(idx, lines[idx]);
                continue;
            }

            parent = false;
        }

        if tag_match.is_match(lines[idx]) {
            let mut hp = String::new();
            for _j in 0..indent {
                hp.push_str("  ");
            }
            print!("{}", hp);
            dump_line(idx, lines[idx]);
            parent = true;
        }
    }
}

fn dump_section_titles(lines: &Vec<&str>) {
    for i in 0..lines.len() {
        let caps = match HDR_LINE.captures(lines[i]) {
            None => continue,
            Some(c) => c,
        };
        let mut hp = String::new();
        for _i in 1..caps.get(1).unwrap().as_str().len() {
            hp.push_str("  ");
        }
        hp.push_str(caps.get(2).unwrap().as_str());
        println!("{}", hp);
    }
}

fn get_section(lines: &Vec<&str>, section: &str) -> (bool, i32, i32) {
    let mut start_idx: i32 = -1;
    let mut end_idx: i32 = -1;
    //let hdr = format!(r"^(#+) .*(?i){}(?-i)($|\s+.*$)", section);
    let hdr = format!(r"^(#+) .*(?i){}(?-i).*$", section);
    let section_hdr = Regex::new(&hdr).unwrap();

    if section == "" {
        return (true, 0, (lines.len() - 1) as i32);
    }

    let mut sec_level = 0;

    for i in 0..lines.len() {
        let caps = match section_hdr.captures(lines[i]) {
            None => continue,
            Some(c) => c,
        };
        sec_level = caps.get(1).unwrap().as_str().len();
        start_idx = i as i32;
        break;
    }

    if start_idx == -1 {
        return (false, -1, -1);
    }

    for i in (start_idx + 1) as usize..lines.len() {
        let caps = match HDR_LINE.captures(lines[i]) {
            None => continue,
            Some(c) => c,
        };
        if sec_level == caps.get(1).unwrap().as_str().len() {
            end_idx = (i - 1) as i32;
            break;
        }
    }

    if end_idx == -1 {
        end_idx = lines.len() as i32 - 1
    }

    return (true, start_idx, end_idx);
}

fn write_file(file: &str, lines: &Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    match fs::write(file, lines.join("\n")) {
        Err(e) => Err(e.to_string())?,
        _ => Ok(()),
    }
}

/*
fn replace_nth_char_safe(s: &str, idx: usize, newchar: char) -> String {
    s.chars().enumerate().map(|(i,c)|
                              if i == idx { newchar }
                              else { c }).collect()
}
*/

fn toggle_task(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 3 {
                if c == ' ' {
                    'x'
                } else {
                    ' '
                }
            } else {
                c
            }
        })
        .collect()
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("h", "", "print this help menu");
    opts.optopt("f", "", "todo file", "<file.md>");
    opts.optflag("l", "", "list section titles");
    opts.optopt("s", "", "task section (default is all tasks)", "<section>");
    opts.optopt("t", "", "show tag", "<tag>");
    opts.optflag("c", "", "show completed");
    opts.optopt("n", "", "new task", "<task>");
    opts.optopt("x", "", "toggle completed", "<task#>");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => return Err(f.to_string())?,
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    let todo_file;
    if matches.opt_present("f") {
        todo_file = matches.opt_str("f").unwrap();
    } else {
        todo_file = match env::var("TODO_FILE") {
            Ok(v) => v,
            Err(_e) => return Err("must specify a todo file")?,
        }
    }

    let txt = fs::read_to_string(&todo_file).expect("failed to read todo file");
    //println!("{:?}", txt);

    let mut lines: Vec<&str> = txt.lines().collect();
    let l_len = lines.len();
    if lines[l_len - 1].len() == 0 {
        lines[l_len - 1] = "\n"; /* insert the last newline if removed */
    }

    let section = match matches.opt_present("s") {
        true => matches.opt_str("s").unwrap(),
        false => "".to_string(),
    };

    let (sec, sec_start, sec_end) = get_section(&lines, &section);
    if !sec {
        return Err(format!("section not found ({})", section))?;
    }

    /* dump the section titles */
    if matches.opt_present("l") {
        dump_section_titles(&lines);
        return Ok(());
    }
    /* dump the selected tag for the section */
    else if matches.opt_present("t") {
        dump_tag(
            &lines,
            sec_start,
            sec_end,
            matches.opt_str("t").unwrap(),
            matches.opt_present("c"),
        );
        return Ok(());
    }
    /* add a new task to the section */
    else if matches.opt_present("n") {
        let task = format!("- [ ] {}", matches.opt_str("n").unwrap());

        /* XXX
         * If sec_start point to a header line...
         *     - search forward for:
         *         - next task line
         *             - insert new task right here
         *         - next header
         *             - insert new task two lines before
         *               (figure out lines between headers)
         *         - eof
         *             - insert new task one line before
         *               (figure out lines between header and eof)
         */

        /*
        for i in sec_start..sec_end {

        }
        */

        lines.insert((sec_start + 2) as usize, &task);
        return write_file(&todo_file, &lines);
    }
    /* toggle a task's completion status */
    else if matches.opt_present("x") {
        let task_num = match matches.opt_str("x").unwrap().parse::<i32>() {
            Ok(n) => (n - 1),
            Err(e) => return Err(e.to_string())?,
        };

        /* XXX make sure this line is a task toggle'able line... */
        let toggled_task = toggle_task(&lines[task_num as usize]);
        lines[task_num as usize] = &toggled_task;
        return write_file(&todo_file, &lines);
    }
    /* dump the selected lines in the section */
    else {
        dump_lines(&lines, sec_start, sec_end, matches.opt_present("c"));
        return Ok(());
    }
}
