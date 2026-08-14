/* 12_atomic_alignas.c — _Atomic (qualifier and specifier), _Alignas */
_Atomic int a;                  /* _Atomic as qualifier */
_Atomic(int) b;                 /* _Atomic(type) as specifier */
_Atomic unsigned long c;
const _Atomic int d;
_Alignas(16) int aligned;       /* _Alignas with constant */
_Alignas(double) char buf[8];   /* _Alignas with type */
_Atomic(char *) atomic_ptr;
