use std::{
    any::{Any, TypeId, type_name},
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
};

pub trait Column: Debug + Clone + Hash + Eq + 'static {
    type Entry: 'static;
}

enum ActionEntry {
    WriteCtx { into: TypeId },
    PopCtx { from: TypeId },
    WriteVal { into: TypeId },
}

pub struct Contexter {
    entries: HashMap<TypeId, Box<dyn Any>>,
    action_index: Vec<ActionEntry>,
}

pub struct ContextHandle<'contexter> {
    contexter: &'contexter mut Contexter,
    inserted_to: TypeId,
}

impl<'contexter> Drop for ContextHandle<'contexter> {
    fn drop(&mut self) {
        self.contexter.action_index.push(ActionEntry::PopCtx {
            from: self.inserted_to,
        });
    }
}

impl Contexter {
    pub fn new() -> Contexter {
        Contexter {
            entries: HashMap::new(),
            action_index: vec![],
        }
    }

    fn get_column<'a, C: Column>(&'a self) -> Option<&'a Vec<C::Entry>> {
        match self.entries.get(&TypeId::of::<C>()) {
            Some(vec) => Some(vec.downcast_ref::<Vec<C::Entry>>().expect(&format!(
                "could not downcast data of {} to a {}",
                type_name::<C>(),
                type_name::<Vec<C::Entry>>(),
            ))),
            None => None,
        }
    }

    fn get_column_mut<'a, C: Column>(&'a mut self) -> Option<&'a mut Vec<C::Entry>> {
        match self.entries.get_mut(&TypeId::of::<C>()) {
            Some(vec) => Some(vec.downcast_mut::<Vec<C::Entry>>().expect(&format!(
                "could not downcast data of {} to a {}",
                type_name::<C>(),
                type_name::<Vec<C::Entry>>(),
            ))),
            None => None,
        }
    }

    pub fn write_to<C: Column>(&mut self, val: C::Entry) {
        match self.get_column_mut::<C>() {
            Some(data) => {
                data.push(val);
            }
            None => {
                self.entries.insert(TypeId::of::<C>(), Box::new(vec![val]));
            }
        }
        self.action_index.push(ActionEntry::WriteVal {
            into: TypeId::of::<C>(),
        });
    }

    #[must_use]
    pub fn add_ctx_to<C: Column>(&mut self, ctx: C::Entry) -> ContextHandle {
        match self.get_column_mut::<C>() {
            Some(data) => {
                data.push(ctx);
            }
            None => {
                self.entries.insert(TypeId::of::<C>(), Box::new(vec![ctx]));
            }
        }
        self.action_index.push(ActionEntry::WriteCtx {
            into: TypeId::of::<C>(),
        });
        ContextHandle {
            contexter: self,
            inserted_to: TypeId::of::<C>(),
        }
    }
}

pub trait ContexterHandle {
    fn write_to<C: Column>(&mut self, val: C::Entry);
    fn add_ctx_to<C: Column>(&mut self, ctx: C::Entry) -> ContextHandle;
}

impl ContexterHandle for Contexter {
    fn write_to<C: Column>(&mut self, val: C::Entry) {
        self.write_to::<C>(val);
    }

    fn add_ctx_to<C: Column>(&mut self, ctx: C::Entry) -> ContextHandle {
        self.add_ctx_to::<C>(ctx)
    }
}

impl<'contexter> ContexterHandle for ContextHandle<'contexter> {
    fn write_to<C: Column>(&mut self, val: C::Entry) {
        self.contexter.write_to::<C>(val);
    }

    fn add_ctx_to<C: Column>(&mut self, ctx: C::Entry) -> ContextHandle {
        self.contexter.add_ctx_to::<C>(ctx)
    }
}

pub struct Frame<'contexter> {
    frame: HashMap<TypeId, usize>,
    contexter: &'contexter Contexter,
}

impl<'contexter> Frame<'contexter> {
    pub fn get<C: Column>(&self) -> Option<&C::Entry> {
        let index = self.frame.get(&TypeId::of::<C>())?;
        Some(
            self.contexter
                .get_column::<C>()?
                .get(*index)
                .expect(&format!(
                    "index {} is out of bounds for column {}",
                    index,
                    type_name::<C>()
                )),
        )
    }
}

pub struct FrameIter<'a> {
    contexter: &'a Contexter,
    actions: std::slice::Iter<'a, ActionEntry>,
    frame: HashMap<TypeId, usize>,
    counts: HashMap<TypeId, usize>,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = Frame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(action) = self.actions.next() {
            match action {
                ActionEntry::WriteCtx { into } => {
                    let count = self.counts.entry(*into).or_insert(0);
                    self.frame.insert(*into, *count);
                    *count += 1;
                }
                ActionEntry::PopCtx { from } => {
                    self.frame.remove(from);
                }
                ActionEntry::WriteVal { into } => {
                    let count = self.counts.entry(*into).or_insert(0);
                    self.frame.insert(*into, *count);
                    *count += 1;
                    // Only yield on WriteVal
                    return Some(Frame {
                        frame: self.frame.clone(),
                        contexter: self.contexter,
                    });
                }
            }
        }
        None
    }
}

impl<'a> IntoIterator for &'a Contexter {
    type Item = Frame<'a>;
    type IntoIter = FrameIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        FrameIter {
            contexter: self,
            actions: self.action_index.iter(),
            frame: HashMap::new(),
            counts: HashMap::new(),
        }
    }
}
