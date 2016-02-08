/*
    srtcli
    Utility to playback subtitles on the command line.

    Copyright 2016 Sam Saint-Pettersen.
    Released under the MIT/X11 License.
*/

mod subtitle;
extern crate stopwatch;
extern crate clock;
extern crate clioptions;
extern crate regex;
use subtitle::Subtitle;
use stopwatch::Stopwatch;
use clock::Clock;
use clioptions::CliOptions;
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::thread;
use std::process::exit;

fn parse_unit(unit: &str) -> i64 {
    let n = unit.parse::<i64>().ok();
    let unit = match n {
        Some(unit) => unit as i64,
        None => 0 as i64,
    };
    unit
}

fn clear_text(lines: u32) {
    for _ in 0..lines {
        println!("");
    }
}

fn convert_time_to_ms(timestamp: &str) -> i64 {
    let mut time: i64 = 0;
    let p = Regex::new(r"(\d{2}):(\d{2}):(\d{2}),(\d{3})").unwrap();
    for cap in p.captures_iter(&timestamp) {
        let hours = parse_unit(&cap.at(1).unwrap());
        let minutes = parse_unit(&cap.at(2).unwrap());
        let seconds = parse_unit(&cap.at(3).unwrap());
        let ms = parse_unit(&cap.at(4).unwrap());
        time = (hours * 3600) + (minutes * 60) + seconds;
        time = (time * 1000) + ms;
    }
    time
}

fn is_valid(data: &str) -> bool {
    let mut valid = false;
    let p = Regex::new(r"^\d+$").unwrap();
    if p.is_match(&data) {
        valid = true;
    }
    valid
}

fn parse_subtitles(program: &str, input: &str) -> Vec<Subtitle> {
    let f = File::open(input).unwrap();
    let file = BufReader::new(&f);
    let mut lines: Vec<String> = Vec::new();
    for line in file.lines() {
        let l = line.unwrap().to_string();
        if !l.is_empty() {
            lines.push(l);
        }
    }
    if !is_valid(&lines[0]) {
        display_error(&program, "Input file is not valid subtitles");
    }

    let mut seqs: Vec<i64> = Vec::new();
    let mut starts: Vec<i64> = Vec::new();
    let mut ends: Vec<i64> = Vec::new();
    let mut contents: Vec<String> = Vec::new();
    for l in lines {
        let mut p = Regex::new(r"^\d+$").unwrap();
        let mut is_timecode = false;
        if p.is_match(&l) {
            seqs.push(parse_unit(&l));
            continue;
        }
        p = Regex::new(r"(\d{2}:\d{2}:\d{2},\d{3})\s-->\s(\d{2}:\d{2}:\d{2},\d{3})").unwrap();
        for cap in p.captures_iter(&l) {
            starts.push(convert_time_to_ms(&cap.at(1).unwrap()));
            ends.push(convert_time_to_ms(&cap.at(2).unwrap()));
            is_timecode = true;
            continue;
        }

        p = Regex::new(r"^[\d-A-Za-z'\s]+").unwrap();
        if p.is_match(&l) && !is_timecode {
            contents.push(l.clone());
            continue;
        }
        else {
            contents.push("|".to_string());
        }
    }

    let contents = contents.join("\n");
    let split = contents.split("|");
    let mut text: Vec<String> = Vec::new();
    for c in split {
        if !c.is_empty() {
            text.push(c.to_string());
        }
    }

    let mut subtitles: Vec<Subtitle> = Vec::new();
    for i in 0..seqs.len() {
        subtitles.push(Subtitle::new(seqs[i], starts[i], ends[i], &text[i]));
    }
    subtitles
}

fn playback_subtitles(program: &str, input: &str, use_clock: bool) {
    let subtitles: Vec<Subtitle> = parse_subtitles(&program, &input);
    let runtime = subtitles[subtitles.len() - 1].get_end();
    let mut clock = Clock::new();
    clock.set_time_ms(runtime);
    clear_text(50);
    println!("Playing back: '{}' (Runtime: {} [{}ms])...", input, clock.get_time(), runtime);
    thread::sleep_ms(3000);
    clock.set_time_ms(0);
    println!("{}", clock.get_time());
    clear_text(50);
    let sw = Stopwatch::start_new();
    let mut i: usize = 0;
    let mut in_subtitle = false;
    loop {
        let time = sw.elapsed_ms();
        clock.set_time_ms(time);

        if time % 1000 == 0 && use_clock && !in_subtitle {
            clear_text(40);
            println!("{}", clock.get_time());
        }
        
        if time == subtitles[i].get_start() {
            clear_text(50);
            in_subtitle = true;
            println!("{}", subtitles[i].get_text());
        }
        if time == subtitles[i].get_end() {
            i += 1;
            in_subtitle = false;
            clear_text(50);
        }

        if i == subtitles.len() {
            break;
        }
    }
    exit(0);
}

fn display_version() {
    println!("srtcli v. 1.0");
    exit(0);
}

fn display_usage(program: &str, exit_code: i32) {
    println!("\nsrtcli");
    println!("Utility to playback subtitles on the command line.");
    println!("\nCopyright 2016 Sam Saint-Pettersen.");
    println!("Released under the MIT/X11 License.");
    println!("\nUsage {} -f|--file <file.srt> -c|--clock [-h|--help]", program);
    println!("\n-f|--file: Subtitle file (SubRip Text) to playback.");
    println!("-c|--clock: Display a playback clock.");
    println!("-h|--help: Display this help information and exit.");
    println!("-v|--version: Display version information and exit.");
    exit(exit_code);
}

fn display_error(program: &str, err: &str) {
    println!("Error: {}.", err);
    display_usage(&program, -1);
}

fn main() {
    let cli = CliOptions::new("srtcli");
    let program = cli.get_program();
    let mut input = String::new();
    let mut use_clock = false;

    if cli.get_num() > 1 {
        for (i, a) in cli.get_args().iter().enumerate() {
            match a.trim() {
                "-h" | "--help" => display_usage(&program, 0),
                "-v" | "--version" => display_version(),
                "-f" | "--file" => input = cli.next_argument(i),
                "-c" | "--clock" => use_clock = true,
                _ => continue,
            }
        }

        if input.is_empty() {
            display_error(&program, "No subtitle file specified");
        }

        playback_subtitles(&program, &input, use_clock);
    }
    else {
        display_error(&program, "No options specified");
    }
}
