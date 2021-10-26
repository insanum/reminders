
use std::env;
use std::fs;
use getopts::Options;
use regex::Regex;
use colored::*;

fn tag_color(tag: &str) -> String
{
    match tag {
        "#high"   => tag.red().to_string(),
        "#medium" => tag.yellow().to_string(),
        "#low"    => tag.green().to_string(),
        _         => tag.magenta().to_string()
    }
}

fn dump_line(i: usize, line: &str)
{
    let task = Regex::new(r"^(\s*)(- \[[ |x]\] )(.*)$").unwrap();
    let tag = Regex::new(r"^#.*$").unwrap();

    let c = task.captures(line).unwrap();
    let x = c.get(2).map(|m| match m.as_str() {
                                 "- [x] " => true,
                                 _        => false
                             }).unwrap();

    let words: Vec<&str> =
        c.get(3).map_or("", |m| m.as_str()).split(' ').collect();

    let mut l = String::new();
    l.push_str(c.get(1).map_or("", |m| m.as_str()));

    for i in 0..words.len() {
        if tag.is_match(&words[i]) {
            l.push_str(&tag_color(&words[i]));
        } else {
            l.push_str(words[i]);
        }

        if i != (words.len() - 1) {
            l.push_str(" ");
        }
    }

    println!("{:>3}: {}",
             (i + 1).to_string().cyan(),
             match x {
                 true  => l.as_str().dimmed().italic().strikethrough(),
                 false => l.as_str().clear()
             });
}

fn dump_lines(lines: &Vec<&str>, sec_start: i32, sec_end: i32,
              show_completed: bool)
{
    let rgx = format!(r"^\s*- \[[{}]\] .*$",
                      match show_completed { true => " |x", false => " " });

    let l_match = Regex::new(&rgx).unwrap();

    for i in sec_start..(sec_end + 1) {
        if l_match.is_match(lines[i as usize]) {
            dump_line(i as usize, lines[i as usize]);
        }
    }
}

fn dump_tag(lines: &Vec<&str>, sec_start: i32, sec_end: i32,
            tag: String, show_completed: bool)
{
    let rgx = format!(r"^\s*- \[[{}]\] .*\s+#{}\s?.*$",
                      match show_completed { true => " |x", false => " " },
                      tag);

    let tag_match = Regex::new(&rgx).unwrap();
    let mut parent = false;

    for i in sec_start..(sec_end + 1) {
        let idx = i as usize;
        if parent {
            if lines[idx].starts_with(" ") {
                dump_line(idx, lines[idx]);
                continue;
            }

            parent = false;
        }

        if tag_match.is_match(lines[idx]) {
            dump_line(idx, lines[idx]);
            parent = true;
        }
    }
}

fn dump_section_titles(lines: &Vec<&str>)
{
    let any_hdr = Regex::new(r"^#+ .*").unwrap();

    for i in 0..lines.len() {
        if any_hdr.is_match(lines[i]) {
            println!("{}", lines[i]);
        }
    }
}

fn get_section(lines: &Vec<&str>, section: &str) -> (bool, String, i32, i32)
{
    let mut start_idx: i32 = -1;
    let mut end_idx: i32 = -1;
    let hdr = format!(r"^#+ .*(?i){}(?-i).*", section);
    let section_hdr = Regex::new(&hdr).unwrap();
    let any_hdr = Regex::new(r"^#+ .*").unwrap();
    let blank = Regex::new(r"^\s*$").unwrap();

    if section == "" {
        return (true, "ALL".to_string(), 0, (lines.len() - 1) as i32);
    }

    let mut sec_hdr = String::new();
    for i in 0..lines.len() {
        if section_hdr.is_match(lines[i]) {
            sec_hdr = lines[i].to_string();
            start_idx = (i + 1) as i32;
            break;
        }
    }

    if start_idx == -1 {
        return (false, sec_hdr, -1, -1)
    }

    while start_idx < lines.len() as i32 &&
          blank.is_match(lines[start_idx as usize]) {
        start_idx += 1;
    }

    if start_idx == lines.len() as i32 {
        return (false, sec_hdr, -1, -1)
    }

    for i in start_idx as usize..lines.len() {
        if any_hdr.is_match(lines[i]) {
            end_idx = (i - 1) as i32;
            break;
        }
    }

    if end_idx == -1 {
        end_idx = lines.len() as i32 - 1
    }

    while end_idx > start_idx &&
          blank.is_match(lines[end_idx as usize]) {
        end_idx -= 1;
    }

    return (true, sec_hdr, start_idx, end_idx);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn write_file(file: &str, lines: &Vec<&str>) -> Result<(), Box<dyn std::error::Error>>
{
    match fs::write(file, lines.join("\n")) {
        Err(e) => Err(e.to_string())?,
        _      => Ok(())
    }
}

/*
fn replace_nth_char_safe(s: &str, idx: usize, newchar: char) -> String {
    s.chars().enumerate().map(|(i,c)|
                              if i == idx { newchar }
                              else { c }).collect()
}
*/

fn toggle_task(s: &str) -> String
{
    s.chars().enumerate().map(|(i,c)|
                              if i == 3 {
                                  if c == ' ' { 'x' }
                                  else { ' ' }
                              } else {
                                  c
                              }).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>>
{
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
        Err(f) => return Err(f.to_string())?
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    if !matches.opt_present("f") {
        return Err("must specify the todo file")?;
    }

    let todo_file = matches.opt_str("f").unwrap();

    let txt = fs::read_to_string(&todo_file)
                  .expect("failed to read todo file");
    //println!("{:?}", txt);

    let mut lines: Vec<&str> = txt.lines().collect();
    let l_len = lines.len();
    if lines[l_len - 1].len() == 0 {
        lines[l_len - 1] = "\n"; /* insert the last newline if removed */
    }

    if !matches.opt_present("f") {
        return Err("must specify the todo file")?;
    }

    let section = match matches.opt_present("s") {
                        true  => matches.opt_str("s").unwrap(),
                        false => "".to_string()
                  };

    let (sec, sec_hdr, sec_start, sec_end) =
        get_section(&lines, &section);
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
        let s = sec_hdr.trim_start_matches("#").trim_start_matches(" ");
        println!("{}", format!("{}:", s).bold().underline());
        dump_tag(&lines, sec_start, sec_end, matches.opt_str("t").unwrap(),
                 matches.opt_present("c"));
        return Ok(());
    }
    /* add a new task to the section */
    else if matches.opt_present("n") {
        let task = format!("- [ ] {}", matches.opt_str("n").unwrap());
        lines.insert(sec_start as usize, &task);
        return write_file(&todo_file, &lines);
    }
    /* toggle a task's completion status */
    else if matches.opt_present("x") {
        let task_num =
            match matches.opt_str("x").unwrap().parse::<i32>() {
                Ok(n)  => (n - 1),
                Err(e) => return Err(e.to_string())?
            };

        /* XXX make sure this line is a task toggle'able line... */
        let toggled_task = toggle_task(&lines[task_num as usize]);
        lines[task_num as usize] = &toggled_task;
        return write_file(&todo_file, &lines);
    }
    /* dump the selected lines in the section */
    else {
        let s = sec_hdr.trim_start_matches("#").trim_start_matches(" ");
        println!("{}", format!("{}:", s).bold().underline());
        dump_lines(&lines, sec_start, sec_end, matches.opt_present("c"));
        return Ok(());
    }
}

