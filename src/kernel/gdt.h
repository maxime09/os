#ifndef _GDT_H_
#define _GDT_H_

#include "types.h"

void encodeGDTEntry(u32 entry, u8 *target, u32 base, u32 limit, u8 access, u8 flags);
void initgdt();


#endif