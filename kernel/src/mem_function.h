#ifndef _MEM_FUNCTION_H
#define _MEM_FUNCTION_H

#include <stddef.h>
#include <stdint.h>

void *memcpy(void *dest, const void *src, size_t n);

void *memset(void *s, uint8_t c, size_t n);

void *memmove(void *dest, const void *src, size_t n);

int memcmp(const void *s1, const void *s2, size_t n);

#endif