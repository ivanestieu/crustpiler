/* 11_abstract_declarators.c — abstract declarators in type-name positions */
int a = sizeof(int *);          /* pointer abstract */
int b = sizeof(int **);         /* double pointer */
int c = sizeof(int [3]);        /* array abstract */
int d = sizeof(int (*)[3]);     /* pointer to array — grouping paren */
int e = sizeof(int (*)(int));  /* pointer to function */
int f = (int *)0 == 0;          /* cast with pointer abstract */
int g = (char (*)[10])0 != 0;   /* cast with grouped abstract */
void fn(int *);                 /* unnamed pointer param */
void fn2(int [], char *);       /* unnamed array + pointer params */
void fn3(int (*)(int));        /* unnamed function-pointer param */
int h = _Alignof(long double);
