#include "lib.h"
#include "keymap.h"

char parse_input(unsigned char input){
    if (input >= 88){
        return 0;
    }else{
        return azerty_keymap[(unsigned int)input];
    }
}

void *malloc(uintptr_t size){
    if(size == 0){
        return 0;
    }
    return memalign(size, 16);
}