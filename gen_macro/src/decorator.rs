use syn::{Arm, Block, Expr, FieldValue, ItemFn, Stmt, parse_quote, token};

trait InnerYield {
    fn replace_yield(&mut self);
}

impl InnerYield for Block {
    fn replace_yield(&mut self) {
        self.stmts.replace_yield();
    }
}

impl InnerYield for Stmt {
    fn replace_yield(&mut self) {
        if let Stmt::Expr(expr, semi) = self {
            (expr, semi).replace_yield();
        }
    }
}

impl InnerYield for Expr {
    fn replace_yield(&mut self) {
        let mut semi = None;
        (self, &mut semi).replace_yield();
    }
}

impl InnerYield for Arm {
    fn replace_yield(&mut self) {
        self.guard.replace_yield();
        self.body.replace_yield();
    }
}

impl InnerYield for FieldValue {
    fn replace_yield(&mut self) {
        self.expr.replace_yield();
    }
}

impl<T: InnerYield> InnerYield for Box<T> {
    fn replace_yield(&mut self) {
        let inner: &mut T = self;
        inner.replace_yield();
    }
}

impl<T: InnerYield> InnerYield for Option<T> {
    fn replace_yield(&mut self) {
        if let Some(inner) = self {
            inner.replace_yield();
        }
    }
}

impl<T: InnerYield, P> InnerYield for (P, T) {
    fn replace_yield(&mut self) {
        self.1.replace_yield();
    }
}

impl<T: InnerYield> InnerYield for Vec<T> {
    fn replace_yield(&mut self) {
        self.iter_mut().for_each(|e| e.replace_yield());
    }
}

impl<T: InnerYield, P> InnerYield for syn::punctuated::Punctuated<T, P> {
    fn replace_yield(&mut self) {
        self.iter_mut().for_each(|e| e.replace_yield());
    }
}

impl InnerYield for (&mut Expr, &mut Option<token::Semi>) {
    fn replace_yield(&mut self) {
        match self.0 {
            Expr::Array(expr_array) => expr_array.elems.replace_yield(),
            Expr::Assign(expr_assign) => {
                expr_assign.left.replace_yield();
                expr_assign.right.replace_yield()
            }
            Expr::Async(_) => (),
            Expr::Await(expr_await) => {
                expr_await.base.replace_yield();
                // replace expr.await => { (*__private_state_ptr).replace(expr); ::rust_generator::SuspendOnce::default().await; }
                let inner_expr = &expr_await.base;
                *self.0 = parse_quote! {
                    {
                        (*__private_state_ptr).replace(#inner_expr);
                        ::rust_generator::SuspendOnce::default().await;
                    }
                };
                *self.1 = None;
            }
            Expr::Binary(expr_binary) => {
                expr_binary.left.replace_yield();
                expr_binary.right.replace_yield();
            }
            Expr::Block(expr_block) => expr_block.block.replace_yield(),
            Expr::Break(expr_break) => expr_break.expr.replace_yield(),
            Expr::Call(expr_call) => {
                expr_call.func.replace_yield();
                expr_call.args.replace_yield();
            }
            Expr::Cast(expr_cast) => expr_cast.expr.replace_yield(),
            Expr::Closure(_) => (),
            Expr::Const(_) => (),
            Expr::Continue(_) => (),
            Expr::Field(expr_field) => expr_field.base.replace_yield(),
            Expr::ForLoop(expr_for_loop) => {
                expr_for_loop.expr.replace_yield();
                expr_for_loop.body.replace_yield();
            }
            Expr::Group(expr_group) => expr_group.expr.replace_yield(),
            Expr::If(expr_if) => {
                expr_if.cond.replace_yield();
                expr_if.then_branch.replace_yield();
                expr_if.else_branch.replace_yield();
            }
            Expr::Index(expr_index) => {
                expr_index.expr.replace_yield();
                expr_index.index.replace_yield();
            }
            Expr::Infer(_) => (),
            Expr::Let(expr_let) => expr_let.expr.replace_yield(),
            Expr::Lit(_) => (),
            Expr::Loop(expr_loop) => expr_loop.body.replace_yield(),
            Expr::Macro(_) => (),
            Expr::Match(expr_match) => {
                expr_match.expr.replace_yield();
                expr_match.arms.replace_yield();
            }
            Expr::MethodCall(expr_method_call) => {
                expr_method_call.receiver.replace_yield();
                expr_method_call.args.replace_yield();
            }
            Expr::Paren(expr_paren) => expr_paren.expr.replace_yield(),
            Expr::Path(_) => (),
            Expr::Range(expr_range) => {
                expr_range.start.replace_yield();
                expr_range.end.replace_yield();
            }
            Expr::RawAddr(expr_raw_addr) => expr_raw_addr.expr.replace_yield(),
            Expr::Reference(expr_reference) => expr_reference.expr.replace_yield(),
            Expr::Repeat(expr_repeat) => {
                expr_repeat.expr.replace_yield();
                expr_repeat.len.replace_yield();
            }
            Expr::Return(expr_return) => expr_return.expr.replace_yield(),
            Expr::Struct(expr_struct) => expr_struct.fields.replace_yield(),
            Expr::Try(expr_try) => expr_try.expr.replace_yield(),
            Expr::TryBlock(expr_try_block) => expr_try_block.block.replace_yield(),
            Expr::Tuple(expr_tuple) => expr_tuple.elems.replace_yield(),
            Expr::Unary(expr_unary) => expr_unary.expr.replace_yield(),
            Expr::Unsafe(expr_unsafe) => expr_unsafe.block.replace_yield(),
            Expr::Verbatim(_) => (),
            Expr::While(expr_while) => {
                expr_while.cond.replace_yield();
                expr_while.body.replace_yield();
            }
            Expr::Yield(expr_yield) => {
                expr_yield.expr.replace_yield();
                // replace yield expr => { (*__private_state_ptr).replace(expr); ::rust_generator::SuspendOnce::default().await; }
                let inner_expr = &expr_yield.expr;
                *self.0 = parse_quote! {
                    {
                        (*__private_state_ptr).replace(#inner_expr);
                        ::rust_generator::SuspendOnce::default().await;
                    }
                };
                *self.1 = None;
            }
            _ => (), // currently not supported
        }
    }
}

pub fn recreate_function(ast: &mut ItemFn) {
    ast.block.replace_yield();
    let block = &ast.block;
    *ast.block = parse_quote! {
        {
            unsafe {
                let mut __private_generator = Box::new(::rust_generator::Generator::new());
                let __private_state_ptr = __private_generator.get_state_ptr();
                __private_generator.set_future(Box::pin(async move {
                    #block
                }));
                __private_generator
            }
        }
    };
}
