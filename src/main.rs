extern crate rusqlite;
extern crate time;
extern crate rand;

use time::Timespec;
use rusqlite::Connection;
use std::io;
use std::path::Path;
use std::env;
use std::process;

#[derive(Debug)]
struct Movie {
    id: i32,
    name: String,
    time_created: Timespec,
    opinion: String,
    rating: i32
}

impl Movie {
    fn new(id: i32, name: String, time_created: Timespec, opinion: String, rating: i32) -> Movie {
        Movie {
            id: id,
            name: name,
            time_created: time_created,
            opinion: opinion,
            rating: rating,
        }
    }

    fn get_name(self) -> String {
        self.name
    }

    fn get_opinion(self) -> String {
        self.opinion
    }

    fn get_id(self) -> i32 {
        self.id
    }

    fn get_rating(self) -> i32 {
        self.rating
    }

    fn get_time(self) -> Timespec {
        self.time_created
    }
}

fn main() {
    let conn = Connection::open(Path::new("movies.db")).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS movies (
        id INTEGER UNIQUE NOT NULL,
        name TEXT NOT NULL,
        time_created TEXT NOT NULL,
        opinion TEXT NOT NULL,
        rating INTEGER)", &[]).unwrap();
    let args: Vec<String> = env::args().collect();
    let length = args.len();

    if length == 1 {
        println!("for help, use \"movies help\"");
    }

    for i in &args {
        if i == "help" { print_help(); }
        else if i == "add" { 
            let newMovie = new_movie();
            conn.execute("INSERT INTO movies VALUES ($1, $2, $3, $4, $5)",
                &[&newMovie.id, &newMovie.name, &newMovie.time_created,
                &newMovie.opinion, &newMovie.rating]).unwrap();
            println!("Your movie has been added, it's id is {}", &newMovie.id);
        }
        else if i == "remove" { 
            println!("ID of movie to be removed: (number)");
            let id: i32 = number_input();

            let mut stmt = conn.prepare("SELECT * FROM movies WHERE id=$1").unwrap();
            let mut movie_iter = stmt.query_map(&[&id,], |row| {
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

            println!("Are you sure you want to remove {}?", q.name.trim());

            let resp1 = get_input();
            let resp2 = resp1.to_lowercase();
            let resp = resp2.trim();

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
        else if i == "show" {
            let mut stmt = conn.prepare("SELECT * FROM movies").unwrap();
            let mut movie_iter = stmt.query_map(&[], |row| {
                Movie {
                    id: row.get(0),
                    name: row.get(1),
                    time_created: row.get(2),
                    opinion: row.get(3),
                    rating: row.get(4)
                }
            }).unwrap();

            let mut count = 0;
            let mut movies: Vec<Movie> = vec![];
            for movie in movie_iter {
                movies.push(movie.unwrap());
                count = count + 1;
            }

            println!("Found {} movie(s)", count);

            for z in movies {
                println!("Name:    {}\nOpinion: {}\nRating:  {}\nID:      {}\nTime:    {}\n",
                         z.name.trim(),
                         z.opinion.trim(),
                         z.rating,
                         z.id,
                         time(z.time_created));
            }
        }
        else { continue; }
    }
}

fn new_movie() -> Movie {
    println!("Name of movie:");
    let name: String = get_input();

    println!("Thoughts:");
    let opinion: String = get_input();

    println!("Rating: (number)");
    let rating: i32 = number_input();

    Movie::new(rand::random::<i32>().abs() / 1000, name,
        time::get_time(), opinion, rating)
}

fn time(t: Timespec) -> String {
    let real_time = time::at(t);
    String::from(format!("{}-{}-{} {}:{}", (real_time.tm_year + 1900), real_time.tm_mon,
                 real_time.tm_mday, real_time.tm_hour, real_time.tm_min))
}

fn print_help() {
    println!("help!");
}

fn number_input() -> i32 {
    let n = match get_input().trim().parse::<i32>() {
        Ok(n) => n,
        Err(e) => { println!("Not a number"); 0 },
    };

    n
}

fn get_input() -> String {
    let mut i = String::new();
    let handle = io::stdin();

    match handle.read_line(&mut i) {
        Ok(n) => {},
        Err(e) => println!("What's your problem?")
    }

    i
}
