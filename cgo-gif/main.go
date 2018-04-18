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

func toGoBytes(ptr unsafe.Pointer,length C.int) []byte {
	return C.GoBytes(ptr, length)
}

func get_ff (img []byte,cwidth,cheight *_Ctype_short,imgbuf []byte) {
	rptr := C.CBytes(imgbuf)
	imgsize :=  C.get_first_frame((*_Ctype_uchar)(C.CBytes(img)),C.ulong(len(img)),cwidth,cheight,(*_Ctype_uchar)(rptr))
	fmt.Println(rptr)
	data := toGoBytes(rptr, imgsize)
	ioutil.WriteFile("test1.gif",data, 0644)
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
	get_ff(img,&cwidth,&cheight,imgbuf)
	fmt.Println("width: ", cwidth)
	fmt.Println("height: ", cheight)
}
