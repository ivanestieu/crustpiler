// dot.rs — Graphviz .dot dumper for the full C AST in ast.rs
//
// Usage:
//     let dot = dump_criterion_file(&file);   // or dump_decl / dump_expr / ...
//     std::fs::write("ast.dot", dot).unwrap();
//     // then:  dot -Tpng ast.dot -o ast.png
//
// HOW IT WORKS
// Each AST node becomes one graph node with a unique numeric id. The dumper
// holds a counter (`next_id`) and a string buffer (`out`). For every node it:
//   1. takes an id,
//   2. emits a `id [label="..."]` line,
//   3. recurses into children, emitting `id -> child_id` edges.
// The label carries the node's variant name plus any scalar payload
// (numbers, identifiers) so the rendered tree is self-explanatory.
// =============================================================================

use crate::ast::ast::{Expr, Item};
use crate::ast::decl_specifiers::{AlignmentSpecifier, TypeExpr};
use crate::ast::declarations::{
    Decl, Declaration, Designator, InitDeclarator, InitItem, Initializer,
};
use crate::ast::declarator::Declarator;
use crate::ast::function_def::FunctionDef;
use crate::ast::parameters::ParamDecl;
use crate::ast::span::Spanned;
use crate::ast::statements::{BlockItem, ForInit, Stmt};
use crate::ast::types::{TypeName, TypeSpec};
use crate::criterion::criterion::{
    CriterionAssertion, CriterionBodyItem, CriterionFile, CriterionSuite, CriterionTest,
};

pub struct DotDumper {
    next_id: usize,
    out: String,
}

impl DotDumper {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            out: String::new(),
        }
    }

    // Allocate a fresh node id.
    fn id(&mut self) -> usize {
        let i = self.next_id;
        self.next_id += 1;
        i
    }

    // Emit a node declaration. `shape`/color vary by category for readability.
    fn node(&mut self, id: usize, label: &str, kind: NodeKind) {
        let (shape, color) = kind.style();
        self.out.push_str(&format!(
            "  n{} [label=\"{}\", shape={}, style=filled, fillcolor=\"{}\"];\n",
            id,
            escape(label),
            shape,
            color
        ));
    }

    // Emit an edge parent -> child, optionally labelled (e.g. field name).
    fn edge(&mut self, parent: usize, child: usize, label: &str) {
        if label.is_empty() {
            self.out
                .push_str(&format!("  n{} -> n{};\n", parent, child));
        } else {
            self.out.push_str(&format!(
                "  n{} -> n{} [label=\"{}\", fontsize=9, color=gray40];\n",
                parent,
                child,
                escape(label)
            ));
        }
    }

    // Wrap the accumulated body in a digraph with sane defaults.
    fn finish(self) -> String {
        let mut s = String::new();
        s.push_str("digraph AST {\n");
        s.push_str("  rankdir=TB;\n");
        s.push_str("  node [fontname=\"monospace\", fontsize=10];\n");
        s.push_str("  edge [fontname=\"monospace\"];\n");
        s.push_str(&self.out);
        s.push_str("}\n");
        s
    }

    // ── Criterion layer ───────────────────────────────────────────────────────

    pub fn criterion_file(&mut self, f: &CriterionFile) -> usize {
        let id = self.id();
        self.node(id, "CriterionFile", NodeKind::Crit);
        for item in &f.items {
            let c = self.item(item);
            self.edge(id, c, "item");
        }
        for suite in &f.suites {
            let c = self.suite(suite);
            self.edge(id, c, "suite");
        }
        id
    }

    fn suite(&mut self, s: &CriterionSuite) -> usize {
        let label = match s.timeout {
            Some(t) => format!("Suite '{}'\\ntimeout={}", s.name, t),
            None => format!("Suite '{}'", s.name),
        };
        let id = self.id();
        self.node(id, &label, NodeKind::Crit);
        for t in &s.tests {
            let c = self.test(t);
            self.edge(id, c, "test");
        }
        id
    }

    fn test(&mut self, t: &CriterionTest) -> usize {
        let mut label = format!("Test '{}'", t.name);
        if t.disabled {
            label.push_str("\\n(disabled)");
        }
        if let Some(to) = t.timeout {
            label.push_str(&format!("\\ntimeout={}", to));
        }
        let id = self.id();
        self.node(id, &label, NodeKind::Crit);
        for item in &t.body {
            let c = self.body_item(item);
            self.edge(id, c, "");
        }
        id
    }

    fn body_item(&mut self, b: &CriterionBodyItem) -> usize {
        match b {
            CriterionBodyItem::Assertion(a) => self.assertion(a),
            CriterionBodyItem::Other(stmt) => self.stmt(&stmt.node),
        }
    }

    fn assertion(&mut self, a: &CriterionAssertion) -> usize {
        let label = format!(
            "Assertion\\n{:?}{}",
            a.kind,
            if a.fatal { " (fatal)" } else { " (non-fatal)" }
        );
        let id = self.id();
        self.node(id, &label, NodeKind::Crit);
        for (i, arg) in a.args.iter().enumerate() {
            let c = self.expr(&arg.node);
            self.edge(id, c, &format!("arg{}", i));
        }
        if let Some(msg) = &a.message {
            let c = self.expr(&msg.node);
            self.edge(id, c, "msg");
        }
        id
    }

    // ── Top-level items ───────────────────────────────────────────────────────

    pub fn item(&mut self, it: &Item) -> usize {
        match it {
            Item::FunctionDef(f) => self.function_def(&f),
            Item::Declaration(d) => self.declaration(d),
        }
    }

    fn function_def(&mut self, f: &FunctionDef) -> usize {
        let name = f.name().unwrap_or("<anon>");
        let id = self.id();
        self.node(id, &format!("FunctionDef '{}'", name), NodeKind::Decl);

        let r = self.type_expr(&f.ret);
        self.edge(id, r, "ret");

        let d = self.declarator(&f.declarator);
        self.edge(id, d, "declarator");

        for p in &f.old_style_params {
            let c = self.declaration(p);
            self.edge(id, c, "old_style_param");
        }

        for item in &f.body {
            let c = self.block_item(item);
            self.edge(id, c, "body");
        }
        id
    }

    // ── Declarations ──────────────────────────────────────────────────────────

    pub fn declaration(&mut self, d: &Declaration) -> usize {
        match d {
            Declaration::Normal(decl_spanned) => self.decl(&decl_spanned.node),
            Declaration::StaticAssert(sa_spanned) => {
                let id = self.id();
                self.node(id, "StaticAssert", NodeKind::Decl);
                let c = self.expr(&sa_spanned.node.cond);
                self.edge(id, c, "cond");

                let msg_id = self.id();
                let message = format!("\"{}\"", sa_spanned.node.message.value);
                self.node(msg_id, &message, NodeKind::Expr);
                self.edge(id, msg_id, "message");

                id
            }
        }
    }

    pub fn decl(&mut self, d: &Decl) -> usize {
        let id = self.id();
        self.node(id, "Decl", NodeKind::Decl);

        let s = self.type_expr(&d.specifiers);
        self.edge(id, s, "specifiers");

        for idecl in &d.declarators {
            let c = self.init_declarator(idecl);
            self.edge(id, c, "declarator");
        }
        id
    }

    fn init_declarator(&mut self, d: &InitDeclarator) -> usize {
        let id = self.id();
        self.node(id, "InitDeclarator", NodeKind::Decl);
        let dec = self.declarator(&d.declarator);
        self.edge(id, dec, "");
        if let Some(init) = &d.init {
            let c = self.initializer(init);
            self.edge(id, c, "init");
        }
        id
    }

    fn initializer(&mut self, init: &Initializer) -> usize {
        match init {
            Initializer::Expr(e) => self.expr(e),
            Initializer::List(items) => {
                let id = self.id();
                self.node(id, "InitList", NodeKind::Decl);
                for it in items {
                    let c = self.init_item(it);
                    self.edge(id, c, "");
                }
                id
            }
        }
    }

    fn init_item(&mut self, it: &InitItem) -> usize {
        let id = self.id();
        self.node(id, "InitItem", NodeKind::Decl);
        for d in &it.designators {
            let label = match d {
                Designator::Index(_) => "designator [idx]",
                Designator::Field(_) => "designator .field",
            };
            let did = self.id();
            self.node(did, label, NodeKind::Decl);
            self.edge(id, did, "");
            if let Designator::Index(e) = d {
                let c = self.expr(e);
                self.edge(did, c, "");
            }
        }
        let v = self.initializer(&it.value);
        self.edge(id, v, "value");
        id
    }

    fn declarator(&mut self, d: &Declarator) -> usize {
        let (label, child): (String, Option<&Declarator>) = match d {
            Declarator::Ident(name) => (format!("Ident '{}'", name), None),
            Declarator::Abstract => ("Abstract".to_string(), None),
            Declarator::Pointer { inner, .. } => ("Pointer *".to_string(), Some(inner)),
            Declarator::Array { inner, .. } => ("Array []".to_string(), Some(inner)),
            Declarator::Function { inner, .. } => ("Function ()".to_string(), Some(inner)),
        };
        let id = self.id();
        self.node(id, &label, NodeKind::Type);
        if let Some(inner) = child {
            let c = self.declarator(inner);
            self.edge(id, c, "inner");
        }
        if let Declarator::Pointer { inner, qualifiers } = d {
            for Spanned { node: q, .. } in qualifiers {
                let qid = self.id();
                self.node(qid, &format!("{:?}", q), NodeKind::Type);
                self.edge(id, qid, "qualifier");
            }
        }
        // Array size / function params, if present
        if let Declarator::Array { size, .. } = d {
            use crate::ast::declarator::ArraySize;
            match size {
                ArraySize::Fixed(sz) => {
                    let c = self.expr(sz);
                    self.edge(id, c, "size");
                }
                ArraySize::Vla => {
                    let vid = self.id();
                    self.node(vid, "VLA *", NodeKind::Type);
                    self.edge(id, vid, "");
                }
                ArraySize::None => {}
            }
        }
        if let Declarator::Function {
            params, variadic, ..
        } = d
        {
            let params = if params.is_some() {
                params.clone().unwrap()
            } else {
                Vec::new()
            };
            for (i, p) in params.iter().enumerate() {
                let c = self.param(p);
                self.edge(id, c, &format!("param{}", i));
            }
            if *variadic {
                let vid = self.id();
                self.node(vid, "...", NodeKind::Type);
                self.edge(id, vid, "");
            }
        }
        id
    }

    fn param(&mut self, p: &ParamDecl) -> usize {
        let id = self.id();
        self.node(id, "Param", NodeKind::Type);
        let s = self.type_expr(&p.specifiers);
        self.edge(id, s, "specifiers");
        let d = self.declarator(&p.declarator);
        self.edge(id, d, "");
        id
    }

    // ── Types ─────────────────────────────────────────────────────────────────

    fn type_spec(&mut self, t: &TypeSpec) -> usize {
        match t {
            TypeSpec::Arithmetic(a) => {
                let id = self.id();
                self.node(id, &a.to_c_string(), NodeKind::Type);
                id
            }
            TypeSpec::Void => {
                let id = self.id();
                self.node(id, "void", NodeKind::Type);
                id
            }
            TypeSpec::Bool => {
                let id = self.id();
                self.node(id, "_Bool", NodeKind::Type);
                id
            }
            TypeSpec::Named(n) => {
                let id = self.id();
                self.node(id, &format!("Type {}", n), NodeKind::Type);
                id
            }
            TypeSpec::Struct(s) => {
                let label = format!(
                    "struct {}",
                    s.name.clone().unwrap_or_else(|| "<anon>".into())
                );
                let id = self.id();
                self.node(id, &label, NodeKind::Type);

                if let Some(fields) = &s.fields {
                    for (i, member) in fields.iter().enumerate() {
                        match member {
                            crate::ast::struct_union::StructMember::Field(f) => {
                                let fid = self.id();
                                self.node(fid, &format!("FieldDecl {}", i), NodeKind::Decl);
                                self.edge(id, fid, "member");

                                // field type
                                let t = self.type_expr(&f.type_expr);
                                self.edge(fid, t, "type");

                                // declarators (may be empty for e.g. unnamed structs/members)
                                for (j, fd) in f.declarators.iter().enumerate() {
                                    let did = self.id();
                                    let decl_label = if fd.declarator.is_some() {
                                        format!("FieldDeclarator {}", j)
                                    } else if fd.bit_width.is_some() {
                                        "AnonBitField".to_string()
                                    } else {
                                        "FieldDeclarator".to_string()
                                    };
                                    self.node(did, &decl_label, NodeKind::Decl);
                                    self.edge(fid, did, "decl");

                                    if let Some(d) = &fd.declarator {
                                        let nd = self.declarator(d);
                                        self.edge(did, nd, "declarator");
                                    }
                                    if let Some(bw) = &fd.bit_width {
                                        let bid = self.expr(&bw.node);
                                        self.edge(did, bid, "bits");
                                    }
                                }
                            }
                            crate::ast::struct_union::StructMember::StaticAssert(sa) => {
                                let sid = self.id();
                                self.node(sid, "StaticAssert", NodeKind::Decl);
                                self.edge(id, sid, "member");
                                let c = self.expr(&sa.cond);
                                self.edge(sid, c, "cond");

                                let msg_id = self.id();
                                let message = format!("\"{}\"", sa.message.value);
                                self.node(msg_id, &message, NodeKind::Expr);
                                self.edge(sid, msg_id, "message");
                            }
                        }
                    }
                }

                id
            }
            TypeSpec::Union(s) => {
                let label = format!(
                    "union {}",
                    s.name.clone().unwrap_or_else(|| "<anon>".into())
                );
                let id = self.id();
                self.node(id, &label, NodeKind::Type);

                if let Some(fields) = &s.fields {
                    for (i, member) in fields.iter().enumerate() {
                        match member {
                            crate::ast::struct_union::StructMember::Field(f) => {
                                let fid = self.id();
                                self.node(fid, &format!("FieldDecl {}", i), NodeKind::Decl);
                                self.edge(id, fid, "member");

                                let t = self.type_expr(&f.type_expr);
                                self.edge(fid, t, "type");

                                for (j, fd) in f.declarators.iter().enumerate() {
                                    let did = self.id();
                                    let decl_label = if fd.declarator.is_some() {
                                        format!("FieldDeclarator {}", j)
                                    } else if fd.bit_width.is_some() {
                                        "AnonBitField".to_string()
                                    } else {
                                        "FieldDeclarator".to_string()
                                    };
                                    self.node(did, &decl_label, NodeKind::Decl);
                                    self.edge(fid, did, "decl");

                                    if let Some(d) = &fd.declarator {
                                        let nd = self.declarator(d);
                                        self.edge(did, nd, "declarator");
                                    }
                                    if let Some(bw) = &fd.bit_width {
                                        let bid = self.expr(&bw.node);
                                        self.edge(did, bid, "bits");
                                    }
                                }
                            }
                            crate::ast::struct_union::StructMember::StaticAssert(sa) => {
                                let sid = self.id();
                                self.node(sid, "StaticAssert", NodeKind::Decl);
                                self.edge(id, sid, "member");
                                let c = self.expr(&sa.cond);
                                self.edge(sid, c, "cond");

                                let msg_id = self.id();
                                let message = format!("\"{}\"", sa.message.value);
                                self.node(msg_id, &message, NodeKind::Expr);
                                self.edge(sid, msg_id, "message");
                            }
                        }
                    }
                }

                id
            }
            TypeSpec::Enum(e) => {
                let id = self.id();
                self.node(
                    id,
                    &format!("enum {}", e.name.clone().unwrap_or_else(|| "<anon>".into())),
                    NodeKind::Type,
                );

                if let Some(variants) = &e.variants {
                    for variant in variants {
                        let vid = self.id();
                        self.node(
                            vid,
                            &format!("Enumerator '{}'", variant.name),
                            NodeKind::Decl,
                        );
                        self.edge(id, vid, "variant");

                        if let Some(value) = &variant.value {
                            let val = self.expr(value);
                            self.edge(vid, val, "value");
                        }
                    }
                }

                id
            }
            TypeSpec::Atomic(tn) => {
                let id = self.id();
                self.node(id, "_Atomic", NodeKind::Type);
                let inner = self.type_name(tn);
                self.edge(id, inner, "");
                id
            }
        }
    }

    fn type_expr(&mut self, t: &TypeExpr) -> usize {
        let id = self.id();
        let mut label = String::from("TypeExpr");

        if let Some(sc) = &t.storage {
            label.push_str(&format!("\\nstorage={:?}", sc));
        }
        if t.thread_local {
            label.push_str("\\n_Thread_local");
        }

        self.node(id, &label, NodeKind::Type);

        let s = self.type_spec(&t.type_spec);
        self.edge(id, s, "spec");

        for q in &t.qualifiers {
            let qid = self.id();
            self.node(qid, &format!("{:?}", q), NodeKind::Type);
            self.edge(id, qid, "qualifier");
        }

        for fs in &t.function_specifiers {
            let fsid = self.id();
            self.node(fsid, &format!("{:?}", fs), NodeKind::Type);
            self.edge(id, fsid, "fn_spec");
        }

        if let Some(align) = &t.alignment {
            let aid = self.id();
            self.node(aid, "_Alignas", NodeKind::Type);
            self.edge(id, aid, "alignment");
            match align {
                AlignmentSpecifier::TypeName(tn) => {
                    let c = self.type_name(tn);
                    self.edge(aid, c, "type");
                }
                AlignmentSpecifier::Expr(e) => {
                    let c = self.expr(&e);
                    self.edge(aid, c, "expr");
                }
            }
        }

        id
    }

    // A type-name: specifier-qualifier list + an abstract declarator.
    // The declarator is Abstract-rooted; Abstract means "no derivation".
    fn type_name(&mut self, t: &TypeName) -> usize {
        let id = self.id();
        self.node(id, "TypeName", NodeKind::Type);

        // base type specifier
        let s = self.type_spec(&t.type_expr.type_spec);
        self.edge(id, s, "spec");

        // qualifiers on the base
        for q in &t.type_expr.qualifiers {
            let qid = self.id();
            self.node(qid, &format!("{:?}", q), NodeKind::Type);
            self.edge(id, qid, "qualifier");
        }

        // the abstract declarator chain (*, [], (), or Abstract leaf)
        let d = self.declarator(&t.derived);
        self.edge(id, d, "declarator");

        id
    }

    // ── Statements ────────────────────────────────────────────────────────────

    pub fn stmt(&mut self, s: &Stmt) -> usize {
        match s {
            Stmt::Expr(e) => {
                let id = self.id();
                self.node(id, "ExprStmt", NodeKind::Stmt);
                let c = self.expr(&e.node);
                self.edge(id, c, "");
                id
            }
            Stmt::Empty => {
                let id = self.id();
                self.node(id, "Empty ;", NodeKind::Stmt);
                id
            }
            Stmt::Block(items) => {
                let id = self.id();
                self.node(id, "Block { }", NodeKind::Stmt);
                for it in items {
                    let c = self.block_item(it);
                    self.edge(id, c, "");
                }
                id
            }
            Stmt::If { cond, then, els } => {
                let id = self.id();
                self.node(id, "If", NodeKind::Stmt);
                let c = self.expr(&cond.node);
                self.edge(id, c, "cond");
                let t = self.stmt(&then.node);
                self.edge(id, t, "then");
                if let Some(e) = els {
                    let e = self.stmt(&e.node);
                    self.edge(id, e, "else");
                }
                id
            }
            Stmt::Switch { expr, body } => {
                let id = self.id();
                self.node(id, "Switch", NodeKind::Stmt);
                let e = self.expr(&expr.node);
                self.edge(id, e, "expr");
                let b = self.stmt(&body.node);
                self.edge(id, b, "body");
                id
            }
            Stmt::While { cond, body } => {
                let id = self.id();
                self.node(id, "While", NodeKind::Stmt);
                let c = self.expr(&cond.node);
                self.edge(id, c, "cond");
                let b = self.stmt(&body.node);
                self.edge(id, b, "body");
                id
            }
            Stmt::DoWhile { body, cond } => {
                let id = self.id();
                self.node(id, "DoWhile", NodeKind::Stmt);
                let b = self.stmt(&body.node);
                self.edge(id, b, "body");
                let c = self.expr(&cond.node);
                self.edge(id, c, "cond");
                id
            }
            Stmt::For {
                init,
                cond,
                step,
                body,
            } => {
                let id = self.id();
                self.node(id, "For", NodeKind::Stmt);
                let i = self.for_init(init);
                self.edge(id, i, "init");
                if let Some(c) = cond {
                    let c = self.expr(&c.node);
                    self.edge(id, c, "cond");
                }
                if let Some(s) = step {
                    let s = self.expr(&s.node);
                    self.edge(id, s, "step");
                }
                let b = self.stmt(&body.node);
                self.edge(id, b, "body");
                id
            }
            Stmt::Return(e) => {
                let id = self.id();
                self.node(id, "Return", NodeKind::Stmt);
                if let Some(e) = e {
                    let c = self.expr(&e.node);
                    self.edge(id, c, "");
                }
                id
            }
            Stmt::Break => {
                let id = self.id();
                self.node(id, "Break", NodeKind::Stmt);
                id
            }
            Stmt::Continue => {
                let id = self.id();
                self.node(id, "Continue", NodeKind::Stmt);
                id
            }
            Stmt::Goto(label) => {
                let id = self.id();
                self.node(id, &format!("Goto {}", label), NodeKind::Stmt);
                id
            }
            Stmt::Label(name, inner) => {
                let id = self.id();
                self.node(id, &format!("Label {}:", name), NodeKind::Stmt);
                let c = self.stmt(&inner.node);
                self.edge(id, c, "");
                id
            }
            Stmt::Case(e, inner) => {
                let id = self.id();
                self.node(id, "Case", NodeKind::Stmt);
                let c = self.expr(&e.node);
                self.edge(id, c, "value");
                let s = self.stmt(&inner.node);
                self.edge(id, s, "");
                id
            }
            Stmt::Default(inner) => {
                let id = self.id();
                self.node(id, "Default", NodeKind::Stmt);
                let s = self.stmt(&inner.node);
                self.edge(id, s, "");
                id
            }
        }
    }

    fn block_item(&mut self, b: &BlockItem) -> usize {
        match b {
            BlockItem::Decl(d) => self.declaration(&d),
            BlockItem::Stmt(s) => self.stmt(&s.node),
        }
    }

    fn for_init(&mut self, f: &ForInit) -> usize {
        match f {
            ForInit::Empty => {
                let id = self.id();
                self.node(id, "(empty)", NodeKind::Stmt);
                id
            }
            ForInit::Expr(e) => self.expr(&e.node),
            ForInit::Decl(d) => self.declaration(&d),
        }
    }

    // ── Expressions ───────────────────────────────────────────────────────────

    pub fn expr(&mut self, e: &Expr) -> usize {
        match e {
            Expr::IntLit(lit) => {
                let id = self.id();
                self.node(id, &format!("Int {}", lit.value), NodeKind::Expr);
                id
            }
            Expr::FloatLit(lit) => {
                let id = self.id();
                self.node(id, &format!("Float {}", lit.value), NodeKind::Expr);
                id
            }
            Expr::CharLit(c) => {
                let id = self.id();
                self.node(id, &format!("Char '{}'", c.escape_debug()), NodeKind::Expr);
                id
            }
            Expr::StringLit(s) => {
                let id = self.id();
                self.node(id, &format!("Str \\\"{}\\\"", s.value), NodeKind::Expr);
                id
            }
            Expr::FuncName(name) => {
                let id = self.id();
                self.node(
                    id,
                    &format!(
                        "FuncName {}",
                        if name.is_empty() {
                            "__func__(not bound)"
                        } else {
                            name
                        }
                    ),
                    NodeKind::Expr,
                );
                id
            }
            Expr::Ident(name) => {
                let id = self.id();
                self.node(id, &format!("Ident {}", name), NodeKind::Expr);
                id
            }
            Expr::CompoundLit { type_name, init } => {
                let id = self.id();
                self.node(id, "CompoundLit", NodeKind::Expr);
                let t = self.type_name(type_name);
                self.edge(id, t, "type");
                for it in init {
                    let c = self.init_item(it);
                    self.edge(id, c, "");
                }
                id
            }
            Expr::UnaryOp { op, operand } => {
                let id = self.id();
                self.node(id, &format!("Unary {:?}", op), NodeKind::Expr);
                let c = self.expr(&operand.node);
                self.edge(id, c, "");
                id
            }
            Expr::PostfixOp { op, operand } => {
                let id = self.id();
                self.node(id, &format!("Postfix {:?}", op), NodeKind::Expr);
                let c = self.expr(&operand.node);
                self.edge(id, c, "");
                id
            }
            Expr::BinaryOp { op, lhs, rhs } => {
                let id = self.id();
                self.node(id, &format!("Binary {:?}", op), NodeKind::Expr);
                let l = self.expr(&lhs.node);
                self.edge(id, l, "lhs");
                let r = self.expr(&rhs.node);
                self.edge(id, r, "rhs");
                id
            }
            Expr::Assign { op, lhs, rhs } => {
                let id = self.id();
                self.node(id, &format!("Assign {:?}", op), NodeKind::Expr);
                let l = self.expr(&lhs.node);
                self.edge(id, l, "lhs");
                let r = self.expr(&rhs.node);
                self.edge(id, r, "rhs");
                id
            }
            Expr::Ternary { cond, then, els } => {
                let id = self.id();
                self.node(id, "Ternary ?:", NodeKind::Expr);
                let c = self.expr(&cond.node);
                self.edge(id, c, "cond");
                let t = self.expr(&then.node);
                self.edge(id, t, "then");
                let e = self.expr(&els.node);
                self.edge(id, e, "else");
                id
            }
            Expr::Call { callee, args } => {
                let id = self.id();
                self.node(id, "Call", NodeKind::Expr);
                let c = self.expr(&callee.node);
                self.edge(id, c, "callee");
                for (i, a) in args.iter().enumerate() {
                    let c = self.expr(&a.node);
                    self.edge(id, c, &format!("arg{}", i));
                }
                id
            }
            Expr::Index { array, index } => {
                let id = self.id();
                self.node(id, "Index []", NodeKind::Expr);
                let a = self.expr(&array.node);
                self.edge(id, a, "array");
                let i = self.expr(&index.node);
                self.edge(id, i, "index");
                id
            }
            Expr::Member { expr, field, arrow } => {
                let op = if *arrow { "->" } else { "." };
                let id = self.id();
                self.node(id, &format!("Member {}{}", op, field), NodeKind::Expr);
                let c = self.expr(&expr.node);
                self.edge(id, c, "");
                id
            }
            Expr::Cast { type_name, expr } => {
                let id = self.id();
                self.node(id, "Cast", NodeKind::Expr);
                let t = self.type_name(type_name);
                self.edge(id, t, "type");
                let c = self.expr(&expr.node);
                self.edge(id, c, "");
                id
            }
            Expr::SizeofExpr(e) => {
                let id = self.id();
                self.node(id, "sizeof expr", NodeKind::Expr);
                let c = self.expr(&e.node);
                self.edge(id, c, "operand");
                id
            }
            Expr::SizeofType(t) => {
                let id = self.id();
                self.node(id, "sizeof type", NodeKind::Expr);
                let c = self.type_name(t);
                self.edge(id, c, "type");
                id
            }
            Expr::AlignofType(t) => {
                let id = self.id();
                self.node(id, "_Alignof", NodeKind::Expr);
                let c = self.type_name(t);
                self.edge(id, c, "");
                id
            }
            Expr::Comma(a, b) => {
                let id = self.id();
                self.node(id, "Comma ,", NodeKind::Expr);
                let l = self.expr(&a.node);
                self.edge(id, l, "");
                let r = self.expr(&b.node);
                self.edge(id, r, "");
                id
            }
            Expr::Generic {
                controlling,
                associated,
            } => {
                let id = self.id();
                self.node(id, "_Generic", NodeKind::Expr);
                let c = self.expr(&controlling.node);
                self.edge(id, c, "controlling");
                for (i, assoc) in associated.iter().enumerate() {
                    let aid = self.id();
                    let type_label = if let Some(_) = &assoc.type_name {
                        "GenericAssoc (type)"
                    } else {
                        "GenericAssoc (default)"
                    };
                    self.node(aid, type_label, NodeKind::Expr);
                    self.edge(id, aid, &format!("assoc{}", i));

                    if let Some(tn) = &assoc.type_name {
                        let teid = self.type_name(tn);
                        self.edge(aid, teid, "type");
                    }

                    let vid = self.expr(&assoc.value.node);
                    self.edge(aid, vid, "value");
                }
                id
            }
        }
    }
}

// Node categories drive colour/shape so the rendered graph is readable at a glance.
#[derive(Clone, Copy)]
enum NodeKind {
    Crit,
    Decl,
    Type,
    Stmt,
    Expr,
}

impl NodeKind {
    fn style(self) -> (&'static str, &'static str) {
        match self {
            NodeKind::Crit => ("box", "#ffe0b2"), // orange  — Criterion layer
            NodeKind::Decl => ("box", "#c8e6c9"), // green   — declarations
            NodeKind::Type => ("ellipse", "#bbdefb"), // blue    — types
            NodeKind::Stmt => ("box", "#e1bee7"), // purple  — statements
            NodeKind::Expr => ("ellipse", "#fff9c4"), // yellow  — expressions
        }
    }
}

// Escape characters that would break the DOT label string.
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
    // keep already-escaped \n sequences working: we used "\\n" in labels,
    // which after the backslash-escape above becomes "\\\\n"; undo that.
}

// ── Public entry points ──────────────────────────────────────────────────────

pub fn dump_criterion_file(f: &CriterionFile) -> String {
    let mut d = DotDumper::new();
    d.criterion_file(f);
    d.finish()
}

pub fn dump_decl(decl: &Decl) -> String {
    let mut d = DotDumper::new();
    d.decl(decl);
    d.finish()
}

pub fn dump_stmt(stmt: &Stmt) -> String {
    let mut d = DotDumper::new();
    d.stmt(stmt);
    d.finish()
}

pub fn dump_expr(expr: &Expr) -> String {
    let mut d = DotDumper::new();
    d.expr(expr);
    d.finish()
}
pub fn dump_translation_unit(items: &Vec<Item>) -> String {
    let mut d = DotDumper::new();
    for item in items {
        d.item(item);
    }
    d.finish()
}
