use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    iter::Rev,
};

pub trait ContextId: Debug + Clone + Eq + Hash {}
impl<T: Debug + Clone + Eq + Hash> ContextId for T {}
pub trait Context: Debug + PartialEq {}
impl<T: Debug + PartialEq> Context for T {}
pub trait Value: Debug + Clone {}
impl<T: Debug + Clone> Value for T {}

pub trait StackHandle<'a, Cid: ContextId, C: Context, T: Value> {
    fn push_context(
        &'a mut self,
        id: Cid,
        ctx: C,
    ) -> Result<ContextHandle<'a, Cid, C, T>, ContextError<Cid, C>>;

    fn write(&mut self, value: T);

    fn get_all(&'a self) -> CtxStackIterator<'a, Cid, C, T>;
}

#[derive(Debug, Clone)]
enum Write<Cid: ContextId, C: Context, T: Value> {
    Value(T),
    Ctx(Ctx<Cid, C, T>),
}

#[derive(Debug, Clone)]
pub struct Ctx<Cid: ContextId, C: Context, T: Value> {
    pub id: Cid,
    pub context: C,
    children: Vec<Write<Cid, C, T>>,
}

#[derive(Debug)]
pub struct ContextHandle<'a, Cid: ContextId, C: Context, T: Value> {
    ctx_id: Cid,
    stack: &'a mut CtxStack<Cid, C, T>,
}

impl<'a, Cid: ContextId, C: Context, T: Value> StackHandle<'a, Cid, C, T>
    for ContextHandle<'a, Cid, C, T>
{
    fn push_context(
        &'a mut self,
        id: Cid,
        ctx: C,
    ) -> Result<ContextHandle<'a, Cid, C, T>, ContextError<Cid, C>> {
        self.stack.push_context(id, ctx)
    }

    fn write(&mut self, value: T) {
        self.stack.write(value)
    }

    fn get_all(&'a self) -> CtxStackIterator<'a, Cid, C, T> {
        self.stack.get_all()
    }
}

impl<'a, Cid: ContextId, C: Context, T: Value> Drop for ContextHandle<'a, Cid, C, T> {
    fn drop(&mut self) {
        self.stack.pop_context(&self.ctx_id);
    }
}

#[derive(Debug)]
pub struct CtxStack<Cid: ContextId, C: Context, T: Value> {
    root: Vec<Write<Cid, C, T>>,
    stack: Vec<Ctx<Cid, C, T>>,
    context: HashSet<Cid>,
}

#[derive(Debug, Clone)]
pub enum ContextError<Cid: ContextId, C: Context> {
    ContextOverwrite { id: Cid, ctx: C },
}

impl<Cid: ContextId, C: Context, T: Value> CtxStack<Cid, C, T> {
    pub fn new() -> CtxStack<Cid, C, T> {
        return CtxStack {
            root: vec![],
            stack: vec![],
            context: HashSet::new(),
        };
    }

    #[inline]
    fn tail(&mut self) -> &mut Vec<Write<Cid, C, T>> {
        match &mut self.stack[..] {
            [] => &mut self.root,
            [.., tail] => &mut tail.children,
        }
    }

    fn pop_context(&mut self, ctx_name: &Cid) {
        let Some(pop) = self.stack.pop() else {
            panic!("tried to pop {ctx_name:?} but stack was empty")
        };
        if pop.id != *ctx_name {
            panic!("tried to pop {ctx_name:?} but popped {pop:?}")
        }
        self.tail().push(Write::Ctx(pop));
    }
}

impl<'a, Cid: ContextId, C: Context, T: Value> StackHandle<'a, Cid, C, T> for CtxStack<Cid, C, T> {
    fn push_context(
        &'a mut self,
        id: Cid,
        ctx: C,
    ) -> Result<ContextHandle<'a, Cid, C, T>, ContextError<Cid, C>> {
        if self.context.contains(&id) {
            return Err(ContextError::ContextOverwrite { id, ctx });
        }
        self.context.insert(id.clone());
        let ctx = Ctx {
            id: id.clone(),
            context: ctx,
            children: vec![],
        };
        self.stack.push(ctx);
        Ok(ContextHandle {
            ctx_id: id,
            stack: self,
        })
    }

    fn write(&mut self, value: T) {
        self.tail().push(Write::Value(value));
    }

    fn get_all(&'a self) -> CtxStackIterator<'a, Cid, C, T> {
        CtxStackIterator::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Frame<'a, Cid: ContextId, C: Context, T: Value> {
    pub context: HashMap<&'a Cid, &'a C>,
    pub value: &'a T,
}

struct CtxIterator<'a, Cid: ContextId, C: Context, T: Value> {
    id: &'a Cid,
    ctx: &'a Ctx<Cid, C, T>,
    iter: Rev<<&'a Vec<Write<Cid, C, T>> as IntoIterator>::IntoIter>,
}

pub struct CtxStackIterator<'a, Cid: ContextId, C: Context, T: Value> {
    root_iter: Rev<<&'a Vec<Write<Cid, C, T>> as IntoIterator>::IntoIter>,
    stack_ctx: Vec<CtxIterator<'a, Cid, C, T>>,
}

impl<'a, Cid: ContextId, C: Context, T: Value> CtxStackIterator<'a, Cid, C, T> {
    fn new(stack: &'a CtxStack<Cid, C, T>) -> Self {
        CtxStackIterator {
            root_iter: stack.root.iter().rev(),
            stack_ctx: stack
                .stack
                .iter()
                .map(|ctx| CtxIterator {
                    id: &ctx.id,
                    ctx,
                    iter: ctx.children.iter().rev(),
                })
                .collect(),
        }
    }
}

impl<'a, Cid: ContextId, C: Context, T: Value> Iterator for CtxStackIterator<'a, Cid, C, T> {
    type Item = Frame<'a, Cid, C, T>;

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
                        id: &ctx.id,
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
                        .map(|ctx| (ctx.id, &ctx.ctx.context))
                        .collect(),
                    value: val,
                }),
                Some(Write::Ctx(ctx)) => {
                    self.stack_ctx.push(CtxIterator {
                        id: &ctx.id,
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
