/* 10_complex_expr.c — casts, sizeof, _Alignof, _Generic, member access */
int a = sizeof(int);
int b = sizeof(x);              /* sizeof expr */
int c = _Alignof(double);
int d = (int)3.14;              /* cast */
int e = (unsigned char)256;
int f = sizeof(int) + sizeof(char);
int *g = (int *)0;              /* cast to pointer */
int h = arr[0];                 /* index in initializer */
int i = obj.field;             /* member access */
int j = ptr->field;            /* arrow */
int k = func(1, 2, 3);         /* call */
int l = a++ + ++b;             /* postfix and prefix */
int m = x.y.z;                 /* chained member */
int n = matrix[1][2];          /* chained index */
int o = _Generic(x, int: 1, float: 2, default: 0);   /* _Generic */
int p = a ? b : c ? d : e;     /* nested ternary */
int compound = (int){42};      /* compound literal */
