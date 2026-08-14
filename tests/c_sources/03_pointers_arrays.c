/* 03_pointers_arrays.c — declarator layering */
int *p;
int **pp;
int ***ppp;
int *const cp;                  // const pointer
const int *pc;                  // pointer to const
int *volatile vp;
int a[3];
int b[];                        // incomplete array
int vla[*];                     // variable length array
int m[2][3];                    // multidimensional
int *pa[4];                     // array of pointers
char *s = "hello";
int arr[3] = {1, 2, 3};         // initializer list
int nested[2][2] = {{1, 2}, {3, 4}};
int designated[5] = {[2] = 10, [4] = 20};   // designated initializers
int *const *volatile complex_ptr;
long *lp;
double *dp;
