/* 05_struct_union.c — struct/union specifiers and members */
struct Point {
    int x;
    int y;
};

struct Empty {
    int only;
};

union Value {
    int i;
    float f;
    char bytes[4];
};

struct Nested {
    struct Point origin;
    int width;
    int height;
};

struct WithPointers {
    int *data;
    struct Node *next;
    char **argv;
};

struct MultiDeclarator {
    int a, b, c;
    int *p, **pp;
};

struct Forward;                 /* forward declaration */

struct Bitfields {
    unsigned int flag : 1;
    unsigned int count : 7;
    int : 4;                    /* anonymous bitfield */
    int value : 16;
};

struct AnonMember {
    int tag;
    int payload;
};

struct Var v;                   /* variable of struct type (tag reference) */
