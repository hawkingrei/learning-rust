#include <stdio.h>
#include <stdint.h>
int get_first_frame(unsigned char *ptr, unsigned long length, short *width, short *height, unsigned char *rptr);
void free_first_frame(unsigned char *rptr, unsigned long length);