#ifndef _PRINT_H
#define _PRINT_H 1

#include <stddef.h>
#include <stdarg.h>

void terminal_initialize(void);
void terminal_write(const char* data, size_t size);
void terminal_writestring(const char* data);
void print_uint(unsigned int data);
void print_int(int data);
void vprintf( const char* format, va_list list);
void printf(const char* format, ...);
void scroll_text();

#endif