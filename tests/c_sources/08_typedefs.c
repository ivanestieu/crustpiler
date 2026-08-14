/* 08_typedefs.c — typedef registration and use (exercises Env::is_typedef) */
typedef int MyInt;
typedef unsigned long size_type;
typedef char *String;
typedef int IntArray[10];
typedef struct Point { int x, y;} PointType;
typedef int (*Callback)(int, int);
typedef enum Color ColorType;

/* Uses of the typedefs above — parser must recognize them as types */
MyInt x = 5;
size_type len;
String name;
PointType p;
Callback cb;

/* typedef of a typedef */
typedef MyInt AliasInt;
AliasInt y;

/* multiple typedef declarators — T usable in later declarators of same line */
typedef int T, *PT, TArr[5];
PT ptr;
TArr arr;
