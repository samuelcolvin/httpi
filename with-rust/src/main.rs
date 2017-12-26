#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate postgres;
extern crate serde_json;
extern crate rocket_contrib;
#[macro_use]
extern crate serde_derive;
extern crate time;

use postgres::{Connection, TlsMode};
use rocket_contrib::{Json};
use time::precise_time_s;

const STEPS:i32 = 5;

fn find_pi() -> (f32, f32) {
    let dsn = "postgres://postgres:waffle@localhost:5432";
    let conn = Connection::connect(dsn, TlsMode::None).unwrap();
    let start = precise_time_s();
    let mut a:f32 = 0.0;
    for _ in 0..STEPS {
        let v:bool = conn.query("SELECT |/(random() ^ 2 + random() ^ 2) < 1", &[])
            .unwrap().iter().next().unwrap().get(0);
        a = a + v as i32 as f32;
    }
    let pi:f32 = a / (STEPS as f32) * 4.0;
    let diff = (precise_time_s() - start) * 1000.0;
//    println!("pi={:.4}, time={:.4}", pi, diff);
    return (pi, diff as f32)
}

#[derive(Serialize)]
struct Result {
    pi: f32,
    sql_exec_time: f32,
}

#[get("/")]
fn index() -> Json<Result> {
    let (pi, diff) = find_pi();
    Json(Result {
        pi: pi,
        sql_exec_time: diff,
    })
}

fn main() {
    rocket::ignite()
        .mount("/", routes![index])
        .launch();
}

// for testing
//fn main() {
//    find_pi();
//}