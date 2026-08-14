/* 02_initializers_expr.c — initializer expressions (precedence, unary, calls) */
int a = 1;
int b = 1 + 2 * 3;              /* precedence: * binds tighter */
int c = (1 + 2) * 3;            /* parentheses override */
int d = 1 + 2 + 3 + 4;          /* left-assoc chain */
int e = 10 - 2 - 3;             /* left-assoc: (10-2)-3 */
int f = 1 << 4 | 2 & 3;         /* mixed bitwise precedence */
int g = 1 < 2 && 3 > 2;         /* relational + logical */
int h = 1 == 1 ? 10 : 20;       /* ternary */
int i = -5;                     /* unary minus */
int j = !0;                     /* unary not */
int k = ~0;                     /* bitwise not */
int l = +3;                     /* unary plus */
int m = 1, n = 2, o = 3;        /* comma-separated declarators */
double x = 3.14;
double y = 1.0e10;
char z = 'a';
char nl = '\n';
