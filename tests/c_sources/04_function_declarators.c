/* 04_function_declarators.c — function-type declarators (not definitions) */
int f(int, char);
int g(void);
int h();                        /* empty param list */
int add(int a, int b);
void proc(int x);
int variadic(int fmt, ...);
int (*fp)(void);                /* pointer to function */
int (*signal(int, void (*)(int)))(int);   /* the classic — pointer to func returning ptr to func */
double compute(double x, double y, double z);
char *strdup(const char *s);
void take_ptr(int *p);
void take_array(int a[]);
void take_sized_array(int a[10]);
int apply(int (*fn)(int), int x);   /* function pointer parameter */
