package main

import (
	"log"
	"math/rand"
	"net/http"
	"os"
	"strconv"
	"fmt"
	"github.com/jackc/pgx"
	"github.com/labstack/echo"
)

const STEPS_DEFAULT = 100

var pool *pgx.ConnPool

type Response struct {
	Pi float32 `json:"pi" xml:"pi"`
	Steps int `json:"steps" xml:"steps"`
}

func get_steps(ctx echo.Context) int {
	steps_strs := ctx.QueryParam("steps")
	steps, err := strconv.ParseInt(steps_strs, 10, 64)
	if err != nil {
		steps = STEPS_DEFAULT
	}
	return int(steps)
}

func native(ctx echo.Context) error {
	steps := get_steps(ctx)
	var circ = 0
	for i := 0; i < steps; i++ {
		a := rand.Float32()
		b := rand.Float32()
		if (a*a + b*b) < 1 {
			circ++
		}
	}
	pi := float32(circ) / float32(steps) * 4
	r := &Response{
		Pi: pi,
		Steps: steps,
	}
	return ctx.JSON(http.StatusOK, r)
}

func sql(ctx echo.Context) error {
	steps := get_steps(ctx)

	conn, err := pool.Acquire()
	if err != nil {
		fmt.Fprintln(os.Stderr, "Error acquiring connection:", err)
		return ctx.String(http.StatusServiceUnavailable, "connection not available")
	}
	defer pool.Release(conn)

	var circ = 0
	for i := 0; i < steps; i++ {
		var v bool
		err = conn.QueryRow("r").Scan(&v)
		if err != nil {
			log.Fatal(err)
			return ctx.String(http.StatusInternalServerError, "sql error")
		}
		if v {
			circ += 1
		}
	}
	pi := float32(circ) / float32(steps) * 4
	r := &Response{
		Pi: pi,
		Steps: steps,
	}
	return ctx.JSON(http.StatusOK, r)
}

func main() {
	config, err := pgx.ParseURI(os.Getenv("DB_DSN"))
	if err != nil {
		log.Fatal(err)
		return
	}
	pool_config := pgx.ConnPoolConfig{ConnConfig: config, MaxConnections: 20}
	pool, err = pgx.NewConnPool(pool_config)
	if err != nil {
		log.Fatal(err)
		return
	}
	_, err = pool.Prepare("r", "SELECT (random() ^ 2 + random() ^ 2) < 1")
	if err != nil {
		log.Fatal(err)
		return
	}

	e := echo.New()
	e.GET("/native/", native)
	e.GET("/sql/", sql)
	e.Logger.Fatal(e.Start(":8000"))
}
