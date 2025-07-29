#ifndef __MYOS_LIB_H__
#define __MYOS_LIB_H__
#include <stdint.h>

void print(char *);
void exit(unsigned int);
char input();
void *memalign(uintptr_t size, uintptr_t align);
void *malloc(uintptr_t size);
void free(void *);

char parse_input(unsigned char);


#endif