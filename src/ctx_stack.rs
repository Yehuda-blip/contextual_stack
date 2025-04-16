use std::{
    collections::HashSet,
    fmt::Debug,
    hash::Hash,
    iter::Rev,
};

pub trait Context: Debug + Clone + Hash + Eq {}
impl<T: Debug + Clone + Hash + Eq> Context for T {}
pub trait Value: Debug + Clone {}
impl<T: Debug + Clone> Value for T {}

pub trait StackHandle<'a, C: Context, T: Value> {
    fn push_context(
        &'a mut self,
        ctx: C,
    ) -> Result<ContextHandle<'a, C, T>, ContextError<C>>;

    fn write(&mut self, value: T);

    fn get_all(&'a self) -> CtxStackIterator<'a, C, T>;
}

#[derive(Debug, Clone)]
enum Write<C: Context, T: Value> {
    Value(T),
    Ctx(Ctx<C, T>),
}

#[derive(Debug, Clone)]
pub struct Ctx<C: Context, T: Value> {
    pub context: C,
    children: Vec<Write<C, T>>,
}

#[derive(Debug)]
pub struct ContextHandle<'a, C: Context, T: Value> {
    stack: &'a mut CtxStack<C, T>,
}

impl<'a, C: Context, T: Value> StackHandle<'a, C, T>
    for ContextHandle<'a, C, T>
{
    fn push_context(
        &'a mut self,
        ctx: C,
    ) -> Result<ContextHandle<'a, C, T>, ContextError<C>> {
        self.stack.push_context(ctx)
    }

    fn write(&mut self, value: T) {
        self.stack.write(value)
    }

    fn get_all(&'a self) -> CtxStackIterator<'a, C, T> {
        self.stack.get_all()
    }
}

impl<'a, C: Context, T: Value> Drop for ContextHandle<'a, C, T> {
    fn drop(&mut self) {
        self.stack.pop_context();
    }
}

#[derive(Debug)]
pub struct CtxStack<C: Context, T: Value> {
    root: Vec<Write<C, T>>,
    stack: Vec<Ctx<C, T>>,
    context: HashSet<C>,
}

#[derive(Debug, Clone)]
pub enum ContextError<C: Context> {
    ContextOverwrite { ctx: C },
}

impl<C: Context, T: Value> CtxStack<C, T> {
    pub fn new() -> CtxStack<C, T> {
        return CtxStack {
            root: vec![],
            stack: vec![],
            context: HashSet::new(),
        };
    }

    #[inline]
    fn tail(&mut self) -> &mut Vec<Write<C, T>> {
        match &mut self.stack[..] {
            [] => &mut self.root,
            [.., tail] => &mut tail.children,
        }
    }

    fn pop_context(&mut self) {
        let Some(pop) = self.stack.pop() else {
            panic!("tried to pop a context but stack was empty")
        };
        self.tail().push(Write::Ctx(pop));
    }
}

impl<'a, C: Context, T: Value> StackHandle<'a, C, T> for CtxStack<C, T> {
    fn push_context(
        &'a mut self,
        ctx: C,
    ) -> Result<ContextHandle<'a, C, T>, ContextError<C>> {
        if self.context.contains(&ctx) {
            return Err(ContextError::ContextOverwrite { ctx });
        }
        self.context.insert(ctx.clone());
        let ctx = Ctx {
            context: ctx,
            children: vec![],
        };
        self.stack.push(ctx);
        Ok(ContextHandle {
            stack: self,
        })
    }

    fn write(&mut self, value: T) {
        self.tail().push(Write::Value(value));
    }

    fn get_all(&'a self) -> CtxStackIterator<'a, C, T> {
        CtxStackIterator::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Frame<'a, C: Context, T: Value> {
    pub context: HashSet<&'a C>,
    pub value: &'a T,
}

struct CtxIterator<'a, C: Context, T: Value> {
    ctx: &'a Ctx<C, T>,
    iter: Rev<<&'a Vec<Write<C, T>> as IntoIterator>::IntoIter>,
}

pub struct CtxStackIterator<'a, C: Context, T: Value> {
    root_iter: Rev<<&'a Vec<Write<C, T>> as IntoIterator>::IntoIter>,
    stack_ctx: Vec<CtxIterator<'a, C, T>>,
}

impl<'a, C: Context, T: Value> CtxStackIterator<'a, C, T> {
    fn new(stack: &'a CtxStack<C, T>) -> Self {
        CtxStackIterator {
            root_iter: stack.root.iter().rev(),
            stack_ctx: stack
                .stack
                .iter()
                .map(|ctx| CtxIterator {
                    ctx,
                    iter: ctx.children.iter().rev(),
                })
                .collect(),
        }
    }
}

impl<'a, C: Context, T: Value> Iterator for CtxStackIterator<'a, C, T> {
    type Item = Frame<'a, C, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.stack_ctx[..] {
            [] => match self.root_iter.next() {
                None => None,
                Some(Write::Value(val)) => Some(Frame {
                    context: HashSet::new(),
                    value: val,
                }),
                Some(Write::Ctx(ctx)) => {
                    self.stack_ctx.push(CtxIterator {
                        ctx,
                        iter: ctx.children.iter().rev(),
                    });
                    self.next()
                }
            },
            [.., curr] => match curr.iter.next() {
                Some(Write::Value(val)) => Some(Frame {
                    context: self
                        .stack_ctx
                        .iter()
                        .map(|ctx| (&ctx.ctx.context))
                        .collect(),
                    value: val,
                }),
                Some(Write::Ctx(ctx)) => {
                    self.stack_ctx.push(CtxIterator {
                        ctx,
                        iter: ctx.children.iter().rev(),
                    });
                    self.next()
                }
                None => {
                    self.stack_ctx.pop();
                    self.next()
                }
            },
        }
    }
}
