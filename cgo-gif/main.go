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
	"errors"
)

func roundup(x ,y int) int {
    return ((x + y - 1) / y) * y+1024*1024;
}

func get_first_frame(img []byte) (data []byte, height, width int, err error) {
	cimg := C.CBytes(img)
	defer C.free(unsafe.Pointer(cimg))
	imgbuf := make([]byte, roundup(len(img),64))
	rptr := C.CBytes(imgbuf)
	defer C.free(unsafe.Pointer(rptr))
	cwidth := _Ctype_short(0)
	cheight := _Ctype_short(0)
	imgsize := C.get_first_frame((*_Ctype_uchar)(cimg), C.ulong(len(img)), &cwidth, &cheight, (*_Ctype_uchar)(rptr))
	if imgsize == 0 {
		return []byte{}, 0, 0, errors.New("cannot process this gif")
	}
	width = int(cwidth)
	height = int(cheight)
	data = C.GoBytes(unsafe.Pointer(rptr), (_Ctype_int)(imgsize))
	return
}

func main() {
	img, err := ioutil.ReadFile("test1.gif") // just pass the file name
	if err != nil {
		fmt.Print(err)
		panic(err)
	}
	fmt.Println(len(img))
	//rb := make([]byte,1)
	//rptr := (*_Ctype_char)(unsafe.Pointer(0))
	data,cheight,cwidth,err := get_first_frame(img)
	ioutil.WriteFile("test1.gif",data, 0644)
	fmt.Println("width: ", cwidth)
	fmt.Println("height: ", cheight)
		
	
	
}
