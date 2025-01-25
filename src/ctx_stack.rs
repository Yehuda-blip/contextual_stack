use std::{collections::HashMap, fmt::Debug, iter::Rev};

#[derive(Debug, Clone)]
enum Write<T: Debug + Clone> {
    Value(T),
    Ctx(Ctx<T>),
}

#[derive(Debug, Clone)]
struct Ctx<T: Debug + Clone> {
    name: String,
    value: String,
    children: Vec<Write<T>>,
}

#[derive(Debug)]
pub struct ContextHandle<'a, T: Debug + Clone> {
    ctx_name: String,
    stack: &'a mut CtxStack<T>,
}

impl<'a, T: Debug + Clone> Drop for ContextHandle<'a, T> {
    fn drop(&mut self) {
        self.stack.pop_context(&self.ctx_name);
    }
}

#[derive(Debug)]
pub struct CtxStack<T: Debug + Clone> {
    root: Vec<Write<T>>,
    stack: Vec<Ctx<T>>,
    context: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ContextError {
    ContextOverwrite { name: String, value: String },
}

impl<T: Debug + Clone> CtxStack<T> {
    pub fn new() -> CtxStack<T> {
        return CtxStack {
            root: vec![],
            stack: vec![],
            context: HashMap::new(),
        };
    }

    #[inline]
    fn tail(&mut self) -> &mut Vec<Write<T>> {
        match &mut self.stack[..] {
            [] => &mut self.root,
            [.., tail] => &mut tail.children,
        }
    }

    pub fn push_context<'a>(
        &'a mut self,
        name: String,
        value: String,
    ) -> Result<ContextHandle<'a, T>, ContextError> {
        if self.context.contains_key(&name) {
            return Err(ContextError::ContextOverwrite { name, value });
        }
        self.context.insert(name.clone(), value.clone());
        let ctx = Ctx {
            name: name.clone(),
            value,
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

    pub fn get_all<'a>(&'a self) -> CtxStackIterator<'a, T> {
        CtxStackIterator::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct Frame<'a, T> {
    pub context: HashMap<&'a str, &'a str>,
    pub value: &'a T,
}

struct CtxIterator<'a, T: Debug + Clone> {
    name: &'a str,
    value: &'a str,
    iter: Rev<<&'a Vec<Write<T>> as IntoIterator>::IntoIter>,
}

pub struct CtxStackIterator<'a, T: Debug + Clone> {
    root_iter: Rev<<&'a Vec<Write<T>> as IntoIterator>::IntoIter>,
    stack_ctx: Vec<CtxIterator<'a, T>>,
}

impl<'a, T: Debug + Clone> CtxStackIterator<'a, T> {
    fn new(stack: &'a CtxStack<T>) -> Self {
        CtxStackIterator {
            root_iter: stack.root.iter().rev(),
            stack_ctx: stack
                .stack
                .iter()
                .map(|ctx| CtxIterator {
                    name: &ctx.name,
                    value: &ctx.value,
                    iter: ctx.children.iter().rev(),
                })
                .collect(),
        }
    }
}

impl<'a, T: Debug + Clone> Iterator for CtxStackIterator<'a, T> {
    type Item = Frame<'a, T>;

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
                        value: &ctx.value,
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
                        .map(|ctx| (ctx.name, ctx.value))
                        .collect(),
                    value: val,
                }),
                Some(Write::Ctx(ctx)) => {
                    self.stack_ctx.push(CtxIterator {
                        name: &ctx.name,
                        value: &ctx.value,
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
