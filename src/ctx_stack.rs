use std::{collections::{HashMap, HashSet}, fmt::Debug, hash::Hash, iter::Rev};

pub trait Context: Debug + Hash {}
impl<T: Debug + Hash> Context for T {}
pub trait Value: Debug + Clone {}
impl<T: Debug + Clone> Value for T {}

#[derive(Debug, Clone)]
enum Write<C: Context, T: Value> {
    Value(T),
    Ctx(Ctx<C, T>),
}

#[derive(Debug, Clone)]
pub struct Ctx<C: Context, T: Value> {
    pub name: String,
    pub context: C,
    children: Vec<Write<C, T>>,
}

#[derive(Debug)]
pub struct ContextHandle<'a, C: Context, T: Value> {
    ctx_name: String,
    stack: &'a mut CtxStack<C, T>,
}

impl<'a, C: Context, T: Value> Drop for ContextHandle<'a, C, T> 
    where {
    fn drop(&mut self) {
        self.stack.pop_context(&self.ctx_name);
    }
}

#[derive(Debug)]
pub struct CtxStack<C: Context, T: Value> {
    root: Vec<Write<C, T>>,
    stack: Vec<Ctx<C, T>>,
    context: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum ContextError<C: Context> {
    ContextOverwrite { name: String, ctx: C },
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

    pub(crate) fn push_context<'a>(
        &'a mut self,
        name: String,
        ctx: C
    ) -> Result<ContextHandle<'a, C, T>, ContextError<C>> {
        if self.context.contains(&name) {
            return Err(ContextError::ContextOverwrite { name, ctx });
        }
        self.context.insert(name.clone());
        let ctx = Ctx {
            name: name.clone(),
            context: ctx,
            children: vec![],
        };
        self.stack.push(ctx);
        Ok(ContextHandle {
            ctx_name: name,
            stack: self,
        })
    }

    fn pop_context(&mut self, ctx_name: &str) {
        let Some(pop) = self.stack.pop() else {
            panic!("tried to pop {ctx_name:?} but stack was empty")
        };
        if pop.name != ctx_name {
            panic!("tried to pop {ctx_name:?} but popped {pop:?}")
        }
        self.tail().push(Write::Ctx(pop));
    }

    pub fn write(&mut self, value: T) {
        self.tail().push(Write::Value(value));
    }

    pub fn get_all<'a>(&'a self) -> CtxStackIterator<'a, C, T> {
        CtxStackIterator::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Frame<'a, C: Context, T: Value> {
    pub context: HashMap<&'a str, &'a C>,
    pub value: &'a T,
}

struct CtxIterator<'a, C: Context, T: Value> {
    name: &'a str,
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
                    name: &ctx.name,
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
                    context: HashMap::new(),
                    value: val,
                }),
                Some(Write::Ctx(ctx)) => {
                    self.stack_ctx.push(CtxIterator {
                        name: &ctx.name,
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
                        .map(|ctx| (ctx.name, &ctx.ctx.context))
                        .collect(),
                    value: val,
                }),
                Some(Write::Ctx(ctx)) => {
                    self.stack_ctx.push(CtxIterator {
                        name: &ctx.name,
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
