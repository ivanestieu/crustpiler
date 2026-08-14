/* 09_static_assert.c — _Static_assert at file scope and in structs */
_Static_assert(1, "always true");
_Static_assert(sizeof(int) == 4, "int must be 4 bytes");
_Static_assert(1 + 1 == 2, "arithmetic works");

struct Checked {
    int a;
    _Static_assert(1, "assert inside struct");
    int b;
};

_Static_assert(10 > 5, "comparison");
