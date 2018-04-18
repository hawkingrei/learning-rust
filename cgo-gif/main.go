package main

/*
#cgo LDFLAGS: -L./lib -lrgif 
#include "./lib/librgif.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"io/ioutil"
	"unsafe"
)

func get_ff (img []byte,cwidth,cheight _Ctype_short,rptr unsafe.Pointer) _Ctype_int {
	return C.get_first_frame((*_Ctype_uchar)(C.CBytes(img)),C.ulong(len(img)),&cwidth,&cheight,(*_Ctype_uchar)(rptr))
}

func main() {
	img, err := ioutil.ReadFile("test.gif") // just pass the file name
	if err != nil {
		fmt.Print(err)
		panic(err)
	}
	fmt.Println(len(img))
	//rb := make([]byte,1)
	//rptr := (*_Ctype_char)(unsafe.Pointer(0))
	imgbuf := make([]byte,len(img))
	rptr := C.CBytes(imgbuf)
	cwidth := _Ctype_short(0)
	cheight := _Ctype_short(0)
	fmt.Println(rptr)
	imgsize := get_ff(img,cwidth,cheight,rptr)
	fmt.Println(rptr)
	fmt.Println((_Ctype_int)(imgsize))
	data := C.GoBytes(rptr, imgsize)
	ioutil.WriteFile("test1.gif",data, 0644)
}
