package main

/*
#cgo LDFLAGS: -L./lib -lrgif
#include "./lib/librgif.h"
*/
import "C"
import (
	"fmt"
	"io/ioutil"
	"reflect"
	"unsafe"
)

func main() {
	img, err := ioutil.ReadFile("test.gif") // just pass the file name
	if err != nil {
		fmt.Print(err)
		panic(err)
	}
	fmt.Println(reflect.TypeOf(C.array))
	var charray []byte
	for i := range C.array {
        charray = append(charray, byte(C.array[i]))
        
    }
	fmt.Println(charray)

	fmt.Println(len(img))
	//rb := make([]byte,1)
	//rptr := (*_Ctype_char)(unsafe.Pointer(0))
	xx := make([]byte,len(img))
	rptr := C.CBytes(xx)
	fmt.Println(rptr)
	a := C.get_first_frame((*_Ctype_uchar)(C.CBytes(img)),C.ulong(len(img)),(*_Ctype_uchar)(rptr))
	fmt.Println(rptr)
	//psize := C.ulong(unsafe.Sizeof(rptr))
	fmt.Println("len ",a)
	gbytes :=C.GoBytes(unsafe.Pointer(rptr), (_Ctype_int)(a))
	ioutil.WriteFile("test1.gif",gbytes, 0644)
}
