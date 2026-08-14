/* 07_storage_qualifiers.c — storage classes, qualifiers, specifiers */
static int s_var;
extern int e_var;
register int r_var;
auto int a_var;
static const int sc = 5;
extern const char *msg;
const int ci = 10;
volatile int vi;
const volatile int cvi;
static inline int fast_fn(void);
_Noreturn void die(void);
inline void helper(void);
_Thread_local int tls;
static _Thread_local int static_tls;    /* legal combo */
const int *const cpc = 0;
volatile unsigned long counter;
static struct Point origin;
