# AST → C Example Reference

Each AST type below is paired with the minimal C that produces it. Use these as parser test fixtures.

---

## Literals

### `IntLit` + `IntBase` + `IntSuffix` + `LongKind`
```c
42        // value=42, base=Decimal, suffix={unsigned:false, long:None}
0xFFu     // value=255, base=Hexadecimal, suffix={unsigned:true, long:None}
0777L     // value=511, base=Octal, suffix={unsigned:false, long:Long}
10ULL     // value=10, base=Decimal, suffix={unsigned:true, long:LongLong}
```

### `FloatLit` + `FloatSuffix`
```c
3.14      // value=3.14, suffix=Double
1.0f      // value=1.0,  suffix=Float
2.5L      // value=2.5,  suffix=LongDouble
1e10      // value=1e10, suffix=Double
```

### `StringLit` + `StringPrefix`
```c
"hello"      // value="hello", prefix=None
L"wide"      // prefix=Wide
u8"utf8"     // prefix=Utf8
u"utf16"     // prefix=Utf16
U"utf32"     // prefix=Utf32
```

---

## Types

### `TypeSpec` — primitives
```c
void          // Void
_Bool         // Bool
char          // Char
signed char   // SignedChar
unsigned char // UnsignedChar
short         // Short
unsigned short// UnsignedShort
int           // Int
unsigned int  // UnsignedInt
long          // Long
unsigned long // UnsignedLong
long long     // LongLong
unsigned long long // UnsignedLongLong
float         // Float
double        // Double
long double   // LongDouble
```

### `TypeSpec::Named`
```c
size_t n;        // Named("size_t")
uint8_t b;       // Named("uint8_t")
MyStruct s;      // Named("MyStruct")
```

### `TypeQualifier`
```c
const int x;        // qualifiers=[Const]
volatile int y;     // qualifiers=[Volatile]
int * restrict p;   // restrict on the pointer (Restrict)
const volatile int z; // qualifiers=[Const, Volatile]
```

### `TypeExpr` + `DerivedType`
```c
int                  // spec=Int, derived=[]
int *                // derived=[Pointer([])]
int **               // derived=[Pointer([]), Pointer([])]
int * const          // derived=[Pointer([Const])]
int [3]              // derived=[Array(Some(3))]
int []               // derived=[Array(None)]
int (*)(void)        // derived=[Pointer([]), Function([void], false)]
```

---

## Struct / Union

### `StructOrUnion` + `FieldDecl` + `FieldDeclarator`
```c
// Full definition — fields=Some([...])
struct Point {
    int x;        // FieldDecl{spec:Int, declarators:[FieldDeclarator{Ident("x")}]}
    int y;
};

// Forward declaration — fields=None
struct Node;

// Anonymous — name=None
struct {
    int a;
} anon;

// Union — same shape, TypeSpec::Union
union Value {
    int i;
    float f;
};
```

---

## Enum

### `EnumSpec` + `Enumerator`
```c
// variants=Some([...])
enum Color {
    RED,          // Enumerator{name:"RED", value:None}
    GREEN = 5,    // Enumerator{name:"GREEN", value:Some(5)}
    BLUE          // Enumerator{name:"BLUE", value:None}
};

// Forward / named only — variants=None
enum Color c;
```

---

## Declarators

### `Declarator::Ident`
```c
int x;          // Ident("x")
```

### `Declarator::Abstract`
```c
(int)y;         // cast — the int type has Abstract declarator (no name)
void f(int);    // unnamed param — Abstract
```

### `Declarator::Pointer`
```c
int *p;             // Pointer{qualifiers:[], inner:Ident("p")}
int * const p;      // Pointer{qualifiers:[Const], inner:Ident("p")}
int **pp;           // Pointer{inner:Pointer{inner:Ident("pp")}}
```

### `Declarator::Array`
```c
int a[3];           // Array{inner:Ident("a"), size:Some(3)}
int a[];            // Array{inner:Ident("a"), size:None}
int m[2][3];        // Array{inner:Array{inner:Ident("m"), size:3}, size:2}
```

### `Declarator::Function`
```c
int f(int, char);   // Function{inner:Ident("f"), params:[int,char], variadic:false}
int g(int, ...);    // Function{..., variadic:true}
int (*fp)(void);    // Pointer{inner:Function{inner:Ident("fp"), params:[void]}}
```

---

## Parameters

### `ParamDecl`
```c
void f(int x, const char *s, double);
//      ^^^^^  named param: spec=Int, declarator=Ident("x")
//             ^^^^^^^^^^^^^ spec=Char, qualifiers=[Const], declarator=Pointer(Ident("s"))
//                           ^^^^^^ unnamed: spec=Double, declarator=Abstract
```

---

## Declarations

### `Decl` + `StorageClass`
```c
int x;              // storage=None, spec=Int, declarators=[Ident("x")]
static int y;       // storage=Some(Static)
extern int z;       // storage=Some(Extern)
register int r;     // storage=Some(Register)
auto int a;         // storage=Some(Auto)
typedef int MyInt;  // storage=Some(Typedef)
```

### `InitDeclarator` + multiple declarators
```c
int a, b, c;        // 3 InitDeclarators, all init=None
int x = 5;          // InitDeclarator{declarator:Ident("x"), init:Some(Expr(5))}
int *p = NULL, q;   // 2 declarators: Pointer(p) with init, Ident(q) without
```

### `Initializer::Expr`
```c
int x = 5;          // init = Expr(IntLit(5))
int y = a + b;      // init = Expr(BinaryOp{Add, a, b})
```

### `Initializer::List` + `InitItem`
```c
int arr[3] = {1, 2, 3};
// List([
//   InitItem{designators:[], value:Expr(1)},
//   InitItem{designators:[], value:Expr(2)},
//   InitItem{designators:[], value:Expr(3)},
// ])
```

### `Designator`
```c
int arr[5] = {[2] = 10};
// InitItem{designators:[Index(2)], value:Expr(10)}

struct Point p = {.x = 1, .y = 2};
// InitItem{designators:[Field("x")], value:Expr(1)}
// InitItem{designators:[Field("y")], value:Expr(2)}
```

---

## Expressions

### `Expr::IntLit` / `FloatLit` / `CharLit` / `StringLit`
```c
42          // IntLit
3.14        // FloatLit
'a'         // CharLit('a')
"text"      // StringLit
```

### `Expr::Ident`
```c
x           // Ident("x")
```

### `Expr::CompoundLit`
```c
(struct Point){1, 2}
// CompoundLit{ty:struct Point, init:[InitItem(1), InitItem(2)]}

(int[]){1, 2, 3}
// CompoundLit{ty:int[], init:[...]}
```

### `Expr::UnaryOp` + `UnaryOp`
```c
-x          // UnaryOp{Neg, x}
+x          // UnaryOp{Pos, x}
!x          // UnaryOp{Not, x}
~x          // UnaryOp{BitNot, x}
*p          // UnaryOp{Deref, p}
&x          // UnaryOp{AddrOf, x}
++x         // UnaryOp{PreInc, x}
--x         // UnaryOp{PreDec, x}
```

### `Expr::PostfixOp` + `PostfixOp`
```c
x++         // PostfixOp{PostInc, x}
x--         // PostfixOp{PostDec, x}
```

### `Expr::BinaryOp` + `BinaryOp`
```c
a + b       // BinaryOp{Add, a, b}
a - b       // Sub
a * b       // Mul
a / b       // Div
a % b       // Rem
a & b       // BitAnd
a | b       // BitOr
a ^ b       // BitXor
a << b      // Shl
a >> b      // Shr
a == b      // Eq
a != b      // Ne
a < b       // Lt
a <= b      // Le
a > b       // Gt
a >= b      // Ge
a && b      // And
a || b      // Or
```

### `Expr::Assign` + `AssignOp`
```c
a = b       // Assign{Assign, a, b}
a += b      // AddAssign
a -= b      // SubAssign
a *= b      // MulAssign
a /= b      // DivAssign
a %= b      // RemAssign
a &= b      // BitAndAssign
a |= b      // BitOrAssign
a ^= b      // BitXorAssign
a <<= b     // ShlAssign
a >>= b     // ShrAssign
```

### `Expr::Ternary`
```c
a ? b : c   // Ternary{cond:a, then:b, els:c}
```

### `Expr::Call`
```c
f()         // Call{callee:Ident("f"), args:[]}
f(1, 2)     // Call{callee:Ident("f"), args:[1, 2]}
g(x)(y)     // Call{callee:Call{...}, args:[y]}  — chained
```

### `Expr::Index`
```c
arr[i]      // Index{array:Ident("arr"), index:Ident("i")}
m[i][j]     // Index{array:Index{...}, index:j}
```

### `Expr::Member`
```c
p.x         // Member{expr:p, field:"x", arrow:false}
p->x        // Member{expr:p, field:"x", arrow:true}
a.b.c       // Member{expr:Member{...}, field:"c", arrow:false}
```

### `Expr::Cast`
```c
(int)x          // Cast{ty:int, expr:x}
(char *)ptr     // Cast{ty:char*, expr:ptr}
(double)i / 2   // Cast binds tighter than / — Cast on i only
```

### `Expr::SizeofExpr` / `SizeofType`
```c
sizeof x        // SizeofExpr(x)        — no parens, operand expr
sizeof(x)       // SizeofExpr(x)        — parens around expr
sizeof(int)     // SizeofType(int)      — parens around type
sizeof(int *)   // SizeofType(int*)
```

### `Expr::AlignofType`
```c
_Alignof(int)   // AlignofType(int)
_Alignof(double)// AlignofType(double)
```

### `Expr::Comma`
```c
a, b            // Comma(a, b)
x = (a, b, c)   // Comma(Comma(a,b), c) — value is c
```

---

## Statements

### `Stmt::Expr`
```c
f();        // Expr(Call{...})
x = 5;      // Expr(Assign{...})
i++;        // Expr(PostfixOp{...})
```

### `Stmt::Empty`
```c
;           // Empty
```

### `Stmt::Block` + `BlockItem`
```c
{
    int x = 1;   // BlockItem::Decl
    f(x);        // BlockItem::Stmt
}
```

### `Stmt::If`
```c
if (c) f();              // If{cond:c, then:f(), els:None}
if (c) f(); else g();    // If{cond:c, then:f(), els:Some(g())}
```

### `Stmt::Switch` + `Stmt::Case` + `Stmt::Default`
```c
switch (x) {
    case 1:           // Case(1, ...)
        f();
        break;
    default:          // Default(...)
        g();
}
```

### `Stmt::While`
```c
while (c) f();   // While{cond:c, body:f()}
```

### `Stmt::DoWhile`
```c
do f(); while (c);   // DoWhile{body:f(), cond:c}
```

### `Stmt::For` + `ForInit`
```c
for (;;) f();                // ForInit::Empty, cond:None, step:None
for (i = 0; i < n; i++) f(); // ForInit::Expr
for (int i = 0; i < n; i++) f(); // ForInit::Decl (C99)
```

### `Stmt::Return`
```c
return;     // Return(None)
return x;   // Return(Some(x))
```

### `Stmt::Break` / `Continue` / `Goto`
```c
break;          // Break
continue;       // Continue
goto cleanup;   // Goto("cleanup")
```

### `Stmt::Label`
```c
cleanup:        // Label("cleanup", <next stmt>)
    free(p);
```

---

## Function Definition

### `FunctionDef`
```c
int add(int a, int b) {
    return a + b;
}
// FunctionDef{
//   ret:Int,
//   declarator:Function{inner:Ident("add"), params:[a, b]},
//   body:[Stmt(Return(BinaryOp{Add, a, b}))]
// }

static void helper(void) { }
// storage=Some(Static), ret=Void
```

---

## Top-Level Items

### `Item::FunctionDef` / `Item::Decl`
```c
int global = 0;          // Item::Decl
typedef int Handle;      // Item::Decl (storage=Typedef)

int main(void) {         // Item::FunctionDef
    return 0;
}
```

---

## Criterion Layer

### `CriterionSuite`
```c
TestSuite(math, .timeout = 5.0);
// CriterionSuite{name:"math", timeout:Some(5.0), tests:[...]}
```

### `CriterionTest`
```c
Test(math, addition) {
    cr_assert_eq(add(2, 3), 5);
}
// CriterionTest{suite:"math", name:"addition", disabled:false, body:[...]}

Test(math, skipme, .disabled = true) { }
// disabled=true
```

### `CriterionBodyItem`
```c
Test(math, mixed) {
    int x = compute();        // Other(Stmt::Decl)
    cr_assert(x > 0);         // Assertion
    printf("done\n");         // Other(Stmt::Expr)
}
```

### `CriterionAssertion` + `AssertKind`
```c
cr_assert(x > 0)                    // Assert,  fatal=true
cr_expect(x > 0)                    // Assert,  fatal=false
cr_assert_eq(a, b)                  // Eq
cr_assert_neq(a, b)                 // Ne
cr_assert_lt(a, b)                  // Lt
cr_assert_leq(a, b)                 // Le
cr_assert_gt(a, b)                  // Gt
cr_assert_geq(a, b)                 // Ge
cr_assert_null(p)                   // Null
cr_assert_not_null(p)               // NotNull
cr_assert_float_eq(a, b, 0.001)     // FloatEq
cr_assert_float_neq(a, b, 0.001)    // FloatNe
cr_assert_str_eq(s1, s2)            // StrEq
cr_assert_str_neq(s1, s2)           // StrNe
cr_assert_str_lt(s1, s2)            // StrLt
cr_assert_str_leq(s1, s2)           // StrLe
cr_assert_str_gt(s1, s2)            // StrGt
cr_assert_str_geq(s1, s2)           // StrGe
cr_assert_arr_eq(a, b, n)           // MemEq
cr_assert_arr_neq(a, b, n)          // MemNe
```

### `CriterionAssertion` with message
```c
cr_assert(x == 5, "x should be 5 but was %d", x);
// args:[x == 5], message:Some("x should be 5 but was %d"), plus fmt args
```

---

## Full worked example

```c
#include <criterion/criterion.h>

TestSuite(calculator, .timeout = 2.0);

Test(calculator, addition) {
    int result = add(2, 3);
    cr_assert_eq(result, 5, "2 + 3 should equal 5");
}

Test(calculator, division_by_zero, .disabled = true) {
    cr_assert_eq(divide(10, 0), 0);
}
```

Produces:
```
CriterionFile {
  suites: [
    CriterionSuite {
      name: "calculator",
      timeout: Some(2.0),
      tests: [
        CriterionTest {
          suite: "calculator", name: "addition", disabled: false,
          body: [
            Other(Decl { int result = add(2,3) }),
            Assertion(CriterionAssertion {
              kind: Eq, fatal: true,
              args: [Ident("result"), IntLit(5)],
              message: Some(StringLit("2 + 3 should equal 5")),
            }),
          ],
        },
        CriterionTest {
          suite: "calculator", name: "division_by_zero", disabled: true,
          body: [
            Assertion(CriterionAssertion {
              kind: Eq, fatal: true,
              args: [Call(divide, [10, 0]), IntLit(0)],
              message: None,
            }),
          ],
        },
      ],
    },
  ],
}
```

Translates to Rust:
```rust
#[cfg(test)]
mod calculator {
    use super::*;

    #[test]
    fn addition() {
        let result = add(2, 3);
        assert_eq!(result, 5, "2 + 3 should equal 5");
    }

    #[test]
    #[ignore]
    fn division_by_zero() {
        assert_eq!(divide(10, 0), 0);
    }
}
```