package main

/*
#cgo LDFLAGS: -L../aias-core/go -laias_go
#include <stdlib.h>
#include "../aias-core/go/aias_go.h"
*/
import "C"

import (
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"unsafe"
	"github.com/julienschmidt/httprouter"
	"gorm.io/gorm"
	"gorm.io/driver/sqlite"
)

var (
	privkeyC     *C.char
	pubkeyC      *C.char
	judgePubKeyC *C.char
)

var db *gorm.DB

type SignProcess struct {
	gorm.Model
	M string
	Subset string
}

// SampleServer sends "Hello world" response for requests to /
func SampleServer(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	fmt.Fprintln(w, "Hello world")
}

// Start starts communication with Sender.
// Needs BlindedDigest (m_i) as a request body.
func Start(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprintf(w, "failed to read body")
		return
	}

	bodyC := C.CString(string(body))
	C.new(privkeyC, pubkeyC, judgePubKeyC)

	C.set_blinded_digest(bodyC)

	subset := C.setup_subset()
	subsetGo := C.GoString(subset)

	db.Create(&SignProcess{M: string(body), Subset: subsetGo})

	fmt.Fprintf(w, subsetGo)
}

// Check checks validity of sent parameters
func Check(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	body, err := ioutil.ReadAll(r.Body)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprintf(w, "failed to read body")
		return
	}

	bodyC := C.CString(string(body))
	valid := C.check(bodyC)

	if valid == 0 {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprintf(w, "invalid")
	} else {
		fmt.Fprintf(w, "ok")
	}
}

func main() {
	_db, err := gorm.Open(sqlite.Open("db/db.slite3"), &gorm.Config{})
	db = _db;

	if err != nil {
		panic("failed to connect database")
	}
	   
	db.AutoMigrate(&SignProcess{})

	log.Println("starting server...")

	pubkey := readFile("keys/id_rsa.pub")
	pubkeyC = C.CString(string(pubkey))

	privkey := readFile("keys/id_rsa")
	privkeyC = C.CString(string(privkey))

	judgePubKey := readFile("keys/judge_id_rsa.pub")
	judgePubKeyC = C.CString(string(judgePubKey))

	defer C.destroy()

	defer C.free(unsafe.Pointer(pubkeyC))
	defer C.free(unsafe.Pointer(privkeyC))
	defer C.free(unsafe.Pointer(judgePubKeyC))

	router := httprouter.New()
	router.GET("/", SampleServer)
	router.POST("/start", Start)
	router.POST("/check", Check)

	log.Fatal(http.ListenAndServe(":8080", router))
}

func readFile(file string) []byte {
	f, err := os.Open(file)
	if err != nil {
		log.Fatal("failed to open", file)
	}
	defer f.Close()

	content, err := ioutil.ReadAll(f)
	if err != nil {
		log.Fatal("failed to read", file)
	}

	return content
}
