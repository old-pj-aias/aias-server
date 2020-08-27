package main

import (
	"fmt"
	"log"
	"net/http"

	"github.com/julienschmidt/httprouter"
)

// SampleServer sends "Hello world" response to any incoming requests
func SampleServer(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	fmt.Fprintln(w, "Hello world")
}

func main() {
	router := httprouter.New()

	router.GET("/", SampleServer)

	log.Fatal(http.ListenAndServe(":8080", router))
}
