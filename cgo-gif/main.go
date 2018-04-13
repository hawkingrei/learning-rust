package main

/*
#cgo LDFLAGS: -L./lib -lrgif
#include "./lib/librgif.h"
*/
import "C"
import (
	"fmt"
	"io/ioutil"
)

func main() {
	img, err := ioutil.ReadFile("test.gif") // just pass the file name
	if err != nil {
		fmt.Print(err)
		panic(err)
	}
	fmt.Println(len(img))
	rbyte := C.get_first_frame((*_Ctype_uchar)(C.CBytes(img)),C.ulong(len(img)))
	fmt.Println(rbyte)
}
