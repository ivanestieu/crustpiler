/* 06_enums.c — enum specifiers */
enum Color {
    RED,
    GREEN,
    BLUE
};

enum Status {
    OK = 0,
    WARNING = 5,
    ERROR = 10
};

enum Mixed {
    A,
    B = 100,
    C,                          /* C = 101 (implicit continuation) */
    D = 200
};

enum Flags {
    FLAG_A = 1,
    FLAG_B = 2,
    FLAG_C = 4,
    FLAG_D = 8
};

enum Forward;                   /* named enum, no body */

enum Single { ONLY };

enum Color favorite;            /* variable of enum type */

enum Computed {
    BASE = 1,
    SHIFTED = BASE + 10,        /* value from constant expression */
    DOUBLED = SHIFTED * 2
};
