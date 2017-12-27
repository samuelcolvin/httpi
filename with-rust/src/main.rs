#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate postgres;
extern crate serde_json;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate time;
extern crate rand;

use postgres::{Connection, TlsMode};
use rocket_contrib::{Json};
use time::precise_time_s;
use rand::distributions::{IndependentSample, Range};

const STEPS:i32 = 100;

fn find_pi_sql() -> f64 {
    let dsn = "postgres://postgres:waffle@localhost:5432";
    let conn = Connection::connect(dsn, TlsMode::None).unwrap();
    let mut circ = 0;
    for _ in 0..STEPS {
        let v:bool = conn.query("SELECT (random() ^ 2 + random() ^ 2) < 1", &[])
            .unwrap().iter().next().unwrap().get(0);
        if v {
            circ += 1;
        }
    }
    (circ as f64) / (STEPS as f64) * 4.
}

fn find_pi_fast() -> f64 {
   let between = Range::new(-1f64, 1.);
   let mut rng = rand::thread_rng();

    let mut circ = 0;

    for _ in 0..STEPS {
       let a = between.ind_sample(&mut rng);
       let b = between.ind_sample(&mut rng);
       if a*a + b*b < 1. {
           circ += 1;
       }
   }
    (circ as f64) / (STEPS as f64) * 4.
}

#[derive(Serialize)]
struct Result {
    pi: f64,
}

#[get("/")]
fn index() -> Json<Result> {
    Json(Result {
        pi: find_pi_sql(),
    })
}

#[get("/fast")]
fn fast() -> Json<Result> {
    Json(Result {
        pi: find_pi_fast(),
    })
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, fast])
        .launch();
}

// for testing
//fn main() {
//    let mut start = precise_time_s();
//    let mut pi = find_pi_sql();
//    let mut diff = (precise_time_s() - start) * 1000.0;
//    println!("SQL    pi={:.4} ({:.3}ms)", pi, diff);
//    start = precise_time_s();
//    pi = find_pi_fast();
//    diff = (precise_time_s() - start) * 1000.0;
//    println!("memory pi={:.4} ({:.3}ms)", pi, diff);
//}