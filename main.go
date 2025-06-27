package main

import (
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"strconv"

	"github.com/mnstrapp/mnstrv2server/api/auth"
	"github.com/mnstrapp/mnstrv2server/api/logger"
	"github.com/mnstrapp/mnstrv2server/api/mnstrs/collect"
	"github.com/mnstrapp/mnstrv2server/api/users"
)

var (
	host  string
	port  int
	dbUrl string
)

func init() {
	setupEnv()
}

func main() {
	http.Handle("/api/auth/", logger.NewLogger(auth.NewHandler()))
	http.Handle("/api/users/", logger.NewLogger(users.NewHandler()))
	http.Handle("/api/mnstrs/collect/", logger.NewLogger(collect.NewHandler()))

	conn := fmt.Sprintf("%s:%d", host, port)
	log.Printf("Serving mnstr at %s", conn)
	log.Fatal(http.ListenAndServe(conn, nil))
}

func setupEnv() {
	host = os.Getenv("MNSTR_HOST")
	portStr := os.Getenv("MNSTR_PORT")
	if portStr != "" {
		var err error
		port, err = strconv.Atoi(portStr)
		if err != nil {
			log.Fatal("MNSTR_PORT is not a valid integer")
		}
	}

	dbUrl = os.Getenv("MNSTR_DATABASE_URL")

	flag.StringVar(&host, "host", host, "Host for server")
	flag.IntVar(&port, "port", port, "Port for server")
	flag.StringVar(&dbUrl, "dburl", dbUrl, "URL for database")

	flag.Parse()

	if host == "" {
		log.Fatal("MNSTR_HOST is not set")
	} else {
		os.Setenv("MNSTR_HOST", host)
	}

	if port == 0 {
		log.Fatal("MNSTR_PORT is not set")
	} else {
		os.Setenv("MNSTR_PORT", strconv.Itoa(port))
	}

	if dbUrl == "" {
		log.Fatal("MNSTR_DATABASE_URL is not set")
	} else {
		os.Setenv("MNSTR_DATABASE_URL", dbUrl)
	}
}
