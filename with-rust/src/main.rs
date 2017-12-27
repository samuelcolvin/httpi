#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate postgres;
extern crate serde_json;
extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate rand;
extern crate r2d2;
extern crate r2d2_postgres;

use postgres::{Connection};
use rocket_contrib::{Json};
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rand::distributions::{IndependentSample, Range};
use r2d2_postgres::{TlsMode, PostgresConnectionManager};

const STEPS:i32 = 100;

type Pool = r2d2::Pool<PostgresConnectionManager>;
static DB_DSN: &'static str = env!("DB_DSN");

// Connection request guard type: a wrapper around an r2d2 pooled connection.
pub struct DbConn(pub r2d2::PooledConnection<PostgresConnectionManager>);

/// Attempts to retrieve a single connection from the managed database pool. If
/// no pool is currently managed, fails with an `InternalServerError` status. If
/// no connections are available, fails with a `ServiceUnavailable` status.
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

// For the convenience of using an &DbConn as an &Connection.
impl Deref for DbConn {
    type Target = Connection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn find_pi_sql(conn: &Connection) -> f64 {
    let mut circ = 0i32;
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

    let mut circ = 0i32;
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
fn index(conn: DbConn) -> Json<Result> {
    Json(Result {
        pi: find_pi_sql(&*conn),
    })
}

#[get("/fast")]
fn fast() -> Json<Result> {
    Json(Result {
        pi: find_pi_fast(),
    })
}

fn main() {
    let manager = PostgresConnectionManager::new(DB_DSN, TlsMode::None).unwrap();
    let pool = Pool::builder()
        .max_size(20)
        .build(manager)
        .expect("db pool");
    rocket::ignite()
        .manage(pool)
        .mount("/", routes![index, fast])
        .launch();
}

//// for testing
//extern crate time;
//use time::precise_time_s;
//fn main() {
//    let conn = Connection::connect(DB_DSN, postgres::TlsMode::None).unwrap();
//    let mut start = precise_time_s();
//    let mut pi = find_pi_sql(&conn);
//    let mut diff = (precise_time_s() - start) * 1000.0;
//    println!("SQL    pi={:.4} ({:.3}ms)", pi, diff);
//    start = precise_time_s();
//    pi = find_pi_fast();
//    diff = (precise_time_s() - start) * 1000.0;
//    println!("memory pi={:.4} ({:.3}ms)", pi, diff);
//}
