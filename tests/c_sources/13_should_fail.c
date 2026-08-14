/* 13_should_fail.c — each line is invalid; parser should ERROR on it.
   Test these ONE AT A TIME (they're grouped here for reference).
   Expected: parse_declaration / parse_type_expr returns Err. */

/* signed float          — sign on floating type */
/* void int              — void combined with int */
/* long long long x      — triple long */
/* short long y          — short and long combined */
/* signed unsigned z     — conflicting signs */
/* struct S int w        — tagged type plus arithmetic */
/* _Complex int c        — complex on integer */
/* int x                 — missing semicolon */
/* = 5;                  — missing type and declarator */
/* int 3;                — number where declarator expected */
/* static extern int v   — two storage classes */
