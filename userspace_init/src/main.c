extern void print(char *);
extern void exit(unsigned int);
extern unsigned char input();

void main(){
    print("Hello from user mode\n");
    char *s = malloc(2);
    s[1] = '\0';
    while(1){
        unsigned char i = input();
        if( i != 0){
            if(i == 0xE0 && i == 0xE1){
                i = input();
                continue;
            }
            s[0] = parse_input(i);
            print(s);
        }
    }
}