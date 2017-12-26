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

fn find_pi_sql() -> (f32, f32) {
    let dsn = "postgres://postgres:waffle@localhost:5432";
    let conn = Connection::connect(dsn, TlsMode::None).unwrap();
    let start = precise_time_s();
    let mut circ = 0;
    for _ in 0..STEPS {
        let v:bool = conn.query("SELECT (random() ^ 2 + random() ^ 2) < 1", &[])
            .unwrap().iter().next().unwrap().get(0);
        if v {
            circ += 1;
        }
    }
    let pi:f32 = (circ as f32) / (STEPS as f32) * 4.0;
    let diff = (precise_time_s() - start) * 1000.0;
    return (pi, diff as f32)
}

fn find_pi_fast() -> (f32, f32) {
    let start = precise_time_s();

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
    let pi:f32 = (circ as f32) / (STEPS as f32) * 4.0;
    let diff = (precise_time_s() - start) * 1000.0;
    return (pi, diff as f32)
}

#[derive(Serialize)]
struct Result {
    pi: f32,
    sql_exec_time: f32,
}

#[get("/")]
fn index() -> Json<Result> {
    let (pi, diff) = find_pi_sql();
    Json(Result {
        pi: pi,
        sql_exec_time: diff,
    })
}

#[get("/fast")]
fn fast() -> Json<Result> {
    let (pi, diff) = find_pi_fast();
    Json(Result {
        pi: pi,
        sql_exec_time: diff,
    })
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index, fast])
        .launch();
}

// for testing
//fn main() {
//    let (pi1, diff1) = find_pi_sql();
//    println!("SQL    pi={:.4}, time={:.4}", pi1, diff1);
//    let (pi2, diff2) = find_pi_fast();
//    println!("memory pi={:.4}, time={:.4}", pi2, diff2);
//}