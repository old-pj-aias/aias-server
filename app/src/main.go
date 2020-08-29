package main

/*
#cgo LDFLAGS: -L../core -laias_core
#include "../core/core.h"
*/
import "C"

import (
	"fmt"
	"log"
	"math/big"
	"net/http"

	"github.com/julienschmidt/httprouter"
)

/*
{
    "part_of_encrypted_message": {
        "u": [
            "vector of string"
        ]
    },
    "part_of_unblinder": {
        "r": [
            "vector of int"
        ]
    },
    "part_of_beta": [
        "vector of byte"
    ]
}
*/

const (
	p = big.NewInt(241)
	q = big.NewInt(719)
	n = p.Mul(q)
	e = 23
	k = 30

	// ID is session identifier
	ID = "hoge"
	// S is subset of [1..2k]
	S = []int{1, 4, 7, 8, 10, 11, 12, 14, 17, 18, 22, 23, 24, 26, 30}
)

// EJ is Judge's encryption function
func EJ(m big.Int) (big.Int, error) {
	res := m.Exp(2, nil).Add(1)

	err := nil
	if res == nil {
		err = "failed"
	}

	return res, err
}

// H is one way hash function
func H(m big.Int) (big.Int, error) {
	res := m.Add(1).Exp(2, nil)

	err := nil
	if res == nil {
		err = "failed"
	}

	return res, err
}

// CheckParameter is parameters sent from Sender
type CheckParameter struct {
	PartOfEncryptedMessage EncryptedMessage `json:"part_of_encrypted_message"`
	PartOfUnblinder        Unblinder        `json:"part_of_unblinder"`
	PartOfBeta             []bytes          `json:"part_of_beta"`
}

// EncryptedMessage is m_i where i belongs to S
type EncryptedMessage struct {
	U []string `json:"u"`
}

// Unblinder is r_i where i belongs to S
type Unblinder struct {
	R []big.Int `json:"r"`
}

// SampleServer sends "Hello world" response to any incoming requests
func SampleServer(w http.ResponseWriter, r *http.Request, _ httprouter.Params) {
	fmt.Fprintln(w, "Hello world")
}

func main() {
	log.Println("starting server...")

	router := httprouter.New()

	router.GET("/", SampleServer)

	log.Fatal(http.ListenAndServe(":8080", router))
}
