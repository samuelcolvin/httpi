#![feature(plugin)]
#![feature(custom_derive)]
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

const STEPS_DEFAULT:i32 = 100;

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

fn find_pi_sql(conn: &Connection, steps: i32) -> f64 {
    let mut circ = 0i32;
    let stmt = conn.prepare("SELECT (random() ^ 2 + random() ^ 2) < 1").unwrap();
    for _ in 0..steps {
        let v:bool = stmt.query(&[])
            .unwrap()
            .iter()
            .next()
            .unwrap()
            .get(0);
        if v {
            circ += 1;
        }
    }
    (circ as f64) / (steps as f64) * 4.
}

fn find_pi_native(steps: i32) -> f64 {
    let between = Range::new(-1f64, 1.);
    let mut rng = rand::thread_rng();

    let mut circ = 0i32;
    for _ in 0..steps {
       let a = between.ind_sample(&mut rng);
       let b = between.ind_sample(&mut rng);
       if a*a + b*b < 1. {
           circ += 1;
       }
   }
   (circ as f64) / (steps as f64) * 4.
}

#[derive(Serialize)]
struct Result {
    pi: f64,
}

#[derive(FromForm)]
struct Setup {
    steps: i32
}

#[get("/sql?<setup>")]
fn sql(setup: Setup, conn: DbConn) -> Json<Result> {
    Json(Result {
        pi: find_pi_sql(&*conn, setup.steps),
    })
}

#[get("/sql")]
fn sql_default(conn: DbConn) -> Json<Result> {
    Json(Result {
        pi: find_pi_sql(&*conn, STEPS_DEFAULT),
    })
}

#[get("/native?<setup>")]
fn native(setup: Setup) -> Json<Result> {
    Json(Result {
        pi: find_pi_native(setup.steps),
    })
}

#[get("/native")]
fn native_default() -> Json<Result> {
    Json(Result {
        pi: find_pi_native(STEPS_DEFAULT),
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
        .mount("/", routes![sql, sql_default, native, native_default])
        .launch();
}