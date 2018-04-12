package main

/*
#cgo LDFLAGS: -L./lib -l
#include "./lib/cgif.h"
*/

import "C"

func main() {
	C.get_first_frame(C.CString("John Smith"))
}
