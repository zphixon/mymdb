/* main.rs - Main source file for mymdb
 * (c) 2016 Zack Hixon under MIT license.
 * See LICENSE.txt for more details. */

extern crate rusqlite;
extern crate time;
extern crate rand;
extern crate regex;

use time::Timespec;
use rusqlite::Connection;
use std::io;
use std::path::Path;
use std::env;
use regex::Regex;

// movie struct
#[derive(Debug)]
struct Movie {
    id: i32,
    name: String,
    time_created: Timespec,
    opinion: String,
    rating: i32
}

// movie impl: possibly add search functions
impl Movie {
    // create new movie
    fn new(id: i32, name: String, time_created: Timespec, opinion: String, rating: i32) -> Movie {
        Movie {
            id: id,
            name: name,
            time_created: time_created,
            opinion: opinion,
            rating: rating,
        }
    }
}

fn main() {
    // open connection to .movies.db in current directory: possibly change to
    // ~/.movies.db for simplicity
    let conn = Connection::open(Path::new(".movies.db")).unwrap();

    // create a table in movies database that maps to a movie struct
    conn.execute("CREATE TABLE IF NOT EXISTS movies (
        id INTEGER UNIQUE NOT NULL,
        name TEXT NOT NULL,
        time_created TEXT NOT NULL,
        opinion TEXT NOT NULL,
        rating INTEGER)", &[]).unwrap();

    // get args
    let args: Vec<String> = env::args().collect();
    let length = args.len();

    if length == 1 {
        println!("for help, use \"mymdb help\"");
    }
    else if length == 2 {
        for i in &args {
            if i == "help" { print_help(); }
            else if i == "add" {
                // prompt user for new movie
                let new_movie = new_movie();

                // insert new movie into database
                conn.execute("INSERT INTO movies VALUES ($1, $2, $3, $4, $5)",
                    &[&new_movie.id, &new_movie.name, &new_movie.time_created,
                    &new_movie.opinion, &new_movie.rating]).unwrap();

                println!("Your movie has been added. ID# {}", &new_movie.id);
            }
            else if i == "remove" {
                // get movie id
                println!("ID of movie to be removed: (number)");
                let id: i32 = number_input();

                // find movie by id: TODO: make more efficient
                let mut stmt = conn.prepare("SELECT * FROM movies WHERE id=$1").unwrap();
                let movie_iter = stmt.query_map(&[&id,], |row| {
                    Movie {
                        id: row.get(0),
                        name: row.get(1),
                        time_created: row.get(2),
                        opinion: row.get(3),
                        rating: row.get(4)
                    }
                }).unwrap();

                // get last movie with id because shitty API or I don't know how to use it
                // probably the latter honestly
                let mut q = Movie::new(0, String::new(), time::get_time(), String::new(), 0);
                for i in movie_iter {
                    q = i.unwrap();
                }

                if q.name.is_empty() {
                    println!("Cannot find movie with ID of {}", id);
                    return
                }

                println!("Are you sure you want to remove {}?", q.name.trim());

                let resp = get_input().to_lowercase().trim();

                // remove from database
                if resp == String::from("yes") {
                    conn.execute("DELETE FROM movies WHERE id=$1", &[&id,]).unwrap();
                    println!("{} has been removed from the database.", q.name.trim());
                } else if resp == String::from("y") {
                    conn.execute("DELETE FROM movies WHERE id=$1", &[&id,]).unwrap();
                    println!("{} has been removed from the database.", q.name.trim());
                } else {
                    println!("Movie will not be removed.");
                }
            }
            // show movies in database
            else if i == "show" {
                // select all movies, conver to iterator
                let mut stmt = conn.prepare("SELECT * FROM movies").unwrap();
                let movie_iter = stmt.query_map(&[], |row| {
                    Movie {
                        id: row.get(0),
                        name: row.get(1),
                        time_created: row.get(2),
                        opinion: row.get(3),
                        rating: row.get(4)
                    }
                }).unwrap();

                // convert list of movies to vector
                let mut count = 0;
                let mut movies: Vec<Movie> = vec![];
                for movie in movie_iter {
                    movies.push(movie.unwrap());
                    count = count + 1;
                }

                println!("Found {} movie(s)", count);

                // print movies
                for z in movies {
                    println!("Name:    {}\nOpinion: {}\nRating:  {}\nID:      {}\nTime:    {}\n",
                             z.name.trim(),
                             z.opinion.trim(),
                             z.rating,
                             z.id,
                             time(z.time_created));
                }
            }
            else {
                // TODO: make this not ridiculous
                let re = Regex::new(r"mymdb").unwrap();
                if !re.is_match(&i) {
                    println!("Not an option: {}", i);
                } else {
                    continue;
                }
            }
        }
    }
    else {
        // TODO: get argparse or something like that
        let mut arg3: &String = &String::from("");
        let mut arg4: &String = &String::from("");
        let mut arg5: i32 = 0;
        if length >= 3 {
            arg3 = &args[2];
        }
        if length == 5 {
            arg4 = &args[3];
            arg5 = args[4].trim().parse().expect("3rd arg must be a number");
        }
        for i in &args {
            if i == "-a" {
                // add movie
                let id = rand::random::<i32>().abs() / 1000;
                conn.execute("INSERT INTO movies VALUES ($1, $2, $3, $4, $5)",
                    &[&id, arg3, &(time::get_time()),
                    arg4, &arg5]).unwrap();
                println!("Your movie has been added. ID# {}", id);
            }
            else if i == "-r" {
                // remove movie
                let id: i32 = arg3.trim().parse().expect("arg must be a number");

                let mut stmt = conn.prepare("SELECT * FROM movies WHERE id=$1").unwrap();
                let movie_iter = stmt.query_map(&[&id,], |row| {
                    Movie {
                        id: row.get(0),
                        name: row.get(1),
                        time_created: row.get(2),
                        opinion: row.get(3),
                        rating: row.get(4)
                    }
                }).unwrap();

                let mut q = Movie::new(0, String::new(), time::get_time(), String::new(), 0);
                for i in movie_iter {
                    q = i.unwrap();
                }

                if q.name.is_empty() {
                    println!("Cannot find movie with ID of {}", id);
                    return
                }

                conn.execute("DELETE FROM movies WHERE id=$1", &[&id,]).unwrap();
                println!("{} has been removed from the database.", q.name.trim());
            }
        }
    }
}

// create new movie struct using input
fn new_movie() -> Movie {
    println!("Name of movie:");
    let name: String = get_input();

    println!("Thoughts:");
    let opinion: String = get_input();

    println!("Rating: (number)");
    let rating: i32 = number_input();

    // TODO: non-retarded ID numbers, like autoincrement
    Movie::new(rand::random::<i32>().abs() / 1000, name,
        time::get_time(), opinion, rating)
}

// return time as string
fn time(t: Timespec) -> String {
    let real_time = time::at(t);
    String::from(format!("{}-{}-{} {}:{}", (real_time.tm_year + 1900), real_time.tm_mon,
                 real_time.tm_mday, real_time.tm_hour, real_time.tm_min))
}

fn print_help() {
    println!("mymdb - personal movie database");
    println!("usage:");
    println!("mymdb <command> [options]");
    println!("commands:");
    println!("\tshow - lists movies in database");
    println!("\tadd - adds movie to database (interactively)");
    println!("\tremove - removes movie from database (interactively)");
    println!("options:");
    println!("\t-r <movie ID> - removes movie from database");
    println!("\t-a <movie name> <opinion> <rating>");
    println!("mymdb creates a movie database that the program is run in.");
    println!("It looks for a sqlite file called .movies.db, and if it does");
    println!("not exist, it is created automatically.");
}

// get input as number
fn number_input() -> i32 {
    let n = match get_input().trim().parse::<i32>() {
        Ok(n) => n,
        Err(e) => { println!("Not a number"); 0 },
    };

    n
}

// get input as string
fn get_input() -> String {
    let mut i = String::new();
    let handle = io::stdin();

    match handle.read_line(&mut i) {
        Ok(n) => {},
        Err(e) => println!("What's your problem?")
    }

    i
}
