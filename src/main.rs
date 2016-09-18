/* main.rs - Main source file for mymdb
 * (c) 2016 Zack Hixon under MIT license.
 * See LICENSE.txt for more details. */

extern crate argparse;
extern crate time;
extern crate rusqlite;

use argparse::{ArgumentParser, Store, Print};

use std::io::prelude::*;
use std::io;
use std::env;
use std::str::FromStr;
use std::path::PathBuf;
use std::error::Error;
use time::Timespec;
use rusqlite::Connection;

static VERSION: &'static str = "0.5.1";

struct Movie {
    id: i32,
    name: String,
    time_created: Timespec,
    opinion: String,
    rating: i32,
    version: String
}

enum Command {
    Show,
    Add,
    Remove,
    Edit,
    None
}

impl FromStr for Command {
    type Err = ();
    fn from_str(src: &str) -> Result<Command, ()> {
        return match src {
            "show" => Ok(Command::Show),
            "add" => Ok(Command::Add),
            "remove" => Ok(Command::Remove),
            "edit" => Ok(Command::Edit),
            _ => Err(()),
        };
    }
}

fn main() {
    // open movies database in .movies.db
    let mut path_buf = PathBuf::new();
    path_buf.push(env::home_dir().expect("Could not find home dir!"));
    path_buf.push(".movies.db");
    let path = path_buf.as_path();
    let conn = Connection::open(path).expect("Counld not open database!");

    // create a table in movies database that maps to a movie struct
    conn.execute("CREATE TABLE IF NOT EXISTS movies (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        time_created TEXT NOT NULL,
        opinion TEXT NOT NULL,
        rating INTEGER NOT NULL,
        version TEXT NOT NULL)", &[]).expect("Could not create table!");

    let mut command = Command::None;

    {
        let mut args = ArgumentParser::new();
        args.set_description("Rate your movies");
        args.refer(&mut command)
            .add_argument("command", Store,
                          "Command to run (show, add, remove)").required();
        args.add_option(&["-V", "--version"],
                        Print("mymdb version ".to_string() + VERSION),
                        "Show version of mymdb");
        args.parse_args_or_exit();
    }

    match command {
        Command::Show => {
            // select all movies, convert to iterator
            let mut stmt = conn.prepare("SELECT * FROM movies").expect("Could not prepare statement!");
            let movie_iter = stmt.query_map(&[], |row| {
                Movie {
                    id: row.get(0),
                    name: row.get(1),
                    time_created: row.get(2),
                    opinion: row.get(3),
                    rating: row.get(4),
                    version: row.get(5)
                }
            }).expect("Could not iterate movies!");

            // convert list of movies to vector
            let mut count = 0;
            let mut movies: Vec<Movie> = vec![];
            for movie in movie_iter {
                movies.push(movie.expect("Could not unwrap movie!"));
                count = count + 1;
            }

            println!("Found {} movie(s)", count);

            // print movies
            for z in movies {
                println!("Name:    {}\nOpinion: {}\nRating:  {}\nID:      {}\nTime:    {}\n",
                         z.name,
                         z.opinion,
                         z.rating,
                         z.id,
                         time(z.time_created));
            }
        },
        Command::Add => {
            let new_movie = new_movie();

            match conn.execute("INSERT INTO movies (name, time_created, opinion, rating, version) VALUES ($1, $2, $3, $4, $5)",
                &[&new_movie.name, &new_movie.time_created,
                  &new_movie.opinion, &new_movie.rating, &new_movie.version]) {
                Ok(_) => (),
                Err(e) => {
                    let mut badv = false;
                    let mut stmt = conn.prepare("SELECT version FROM movies")
                        .unwrap();
                    let mut rows = stmt.query(&[]).unwrap();

                    while let Some(row) = rows.next() {
                        let rs: String = row.unwrap().get(0);
                        if rs != VERSION {
                            badv = true;
                        }
                    }

                    if badv {
                        println!("Your movies database contains movies from a different version of mymdb.\nYou may need to recreate your database.");
                    }

                    panic!("{}", Error::description(&e));
                },
            }

            println!("\"{}\" has been added.", new_movie.name);
        },
        Command::Remove => {
            print!("ID of movie to be removed: ");

            let id = get_input_i32();

            let remove = conn.query_row("SELECT * FROM movies WHERE id=$1", &[&id], |row| {
                Movie {
                    id: row.get(0),
                    name: row.get(1),
                    time_created: row.get(2),
                    opinion: row.get(3),
                    rating: row.get(4),
                    version: row.get(5)
                }
            });

            match remove {
                Ok(m) => {
                    print!("Are you sure you want to remove \"{}\"? (yes/no)", m.name);
                    let res: String = get_input().to_lowercase();

                    if res != "yes".to_string() {
                        println!("Will not remove.");
                    } else {
                        conn.execute("DELETE FROM movies WHERE id=$", &[&id,]).unwrap();
                        println!("\"{}\" has been removed from the database.", m.name);
                    }
                },
                Err(_) => println!("Movie does not exist.")
            }
        },
        Command::Edit => {
            print!("ID of movie to be edited: ");

            let old_id = get_input_i32();

            let edit = conn.query_row("SELECT * FROM movies WHERE id=$1", &[&old_id,], |row| {
                Movie {
                    id: row.get(0),
                    name: row.get(1),
                    time_created: row.get(2),
                    opinion: row.get(3),
                    rating: row.get(4),
                    version: row.get(5)
                }
            });

            match edit {
                Ok(_) => {
                    let new_movie = new_movie();

                    match conn.execute("UPDATE movies SET name=$1, time_created=$2, opinion=$3, rating=$4, version=$5 WHERE id=$6",
                        &[&new_movie.name, &new_movie.time_created,
                          &new_movie.opinion, &new_movie.rating,
                          &new_movie.version, &old_id]) {
                        Ok(_) => (),
                        Err(e) => {
                            let mut badv = false;
                            let mut stmt = conn.prepare("SELECT version FROM movies")
                                .unwrap();
                            let mut rows = stmt.query(&[]).unwrap();

                            while let Some(row) = rows.next() {
                                let rs: String = row.unwrap().get(0);
                                if rs != VERSION {
                                    badv = true;
                                }
                            }

                            if badv {
                                println!("Your movies database contains movies from a different version of mymdb.\nYou may need to recreate your database.");
                            }

                            panic!("{}", Error::description(&e));
                        },
                    }

                    println!("\"{}\" has been edited.", new_movie.name);
                },
                Err(_) => println!("Movie does not exist.")
            }
        },
        Command::None => panic!("Incorrect command - argparse or other??")
    }
}

fn new_movie() -> Movie {
    print!("Name of movie: ");
    let name: String = get_input().to_string();

    print!("      Opinion: ");
    let opinion: String = get_input().to_string();

    print!(" Rating (num): ");
    let rating: i32 = get_input_i32();

    Movie {
        id: 0,
        name: name,
        time_created: time::get_time(),
        opinion: opinion,
        rating: rating,
        version: VERSION.into()
    }
}

// get input as string
fn get_input() -> String {
    io::stdout().flush().expect("could not flush stdout");

    let mut i = String::new();
    let handle = io::stdin();

    match handle.read_line(&mut i) {
        Ok(_) => {},
        Err(_) => {
            println!("Could not read input.");
            std::process::exit(3);
        }
    }

    return i.trim().to_string();
}

fn get_input_i32() -> i32 {
    match get_input().parse::<i32>() {
        Ok(n) => n,
        Err(_) => {
            println!("Not a number");
            std::process::exit(2);
        }
    }
}

fn time(t: Timespec) -> String {
    let real_time = time::at(t);
    // might break in 2032 or whenever unix time runs out
    String::from(format!("{}-{}-{} {}:{}", (real_time.tm_year + 1900), real_time.tm_mon,
        real_time.tm_mday, real_time.tm_hour, real_time.tm_min))
}

