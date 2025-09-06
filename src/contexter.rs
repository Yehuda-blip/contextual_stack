use std::{
    any::{Any, TypeId, type_name},
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    vec,
};

use crate::slots::{Slot, Slots};

pub trait Column: Debug + Clone + Hash + Eq + 'static {
    type Entry: 'static;
}

#[macro_export]
macro_rules! column {
    ($name:ident stores $entry:ty) => {
        #[derive(
            std::fmt::Debug, std::clone::Clone, std::hash::Hash, std::cmp::PartialEq, std::cmp::Eq,
        )]
        pub struct $name;

        impl contextual_stack::contexter::Column for $name {
            type Entry = $entry;
        }
    };
}

// 0 duplicate is reserved for the "write"s, so it always appears before any context
const CTX_COUNT_START: usize = 1;
type ColDuplicate = Slot<CTX_COUNT_START>;
const WRITE_SLOT: ColDuplicate = Slots::<CTX_COUNT_START>::reserved(0).unwrap();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActionEntry {
    WriteCtx {
        into: TypeId,
        duplicate: ColDuplicate,
    },
    PopCtx {
        from: TypeId,
        duplicate: ColDuplicate,
    },
    WriteVal {
        into: TypeId,
    },
}

#[derive(Debug)]
pub struct Contexter {
    entries: HashMap<(TypeId, ColDuplicate), Box<dyn Any>>,
    action_index: Vec<ActionEntry>,
    available_duplicates: HashMap<TypeId, Slots<CTX_COUNT_START>>,
}

pub struct ContextHandle<'contexter> {
    contexter: &'contexter mut Contexter,
    inserted_to: (TypeId, ColDuplicate),
}

impl<'contexter> Drop for ContextHandle<'contexter> {
    fn drop(&mut self) {
        let (from, duplicate) = self.inserted_to;
        self.contexter
            .action_index
            .push(ActionEntry::PopCtx { from, duplicate });
        self.contexter.available_duplicates.get_mut(&from).expect(&format!(
            "removed context for {} with duplicate {:?}, but slots for this type were not found",
            type_name::<TypeId>(),
            duplicate
        )).deallocate(duplicate);
    }
}

impl Contexter {
    pub fn new() -> Contexter {
        Contexter {
            entries: HashMap::new(),
            action_index: vec![],
            available_duplicates: HashMap::new(),
        }
    }

    fn get_column<'a, C: Column>(&'a self, duplicate: ColDuplicate) -> Option<&'a Vec<C::Entry>> {
        match self.entries.get(&(TypeId::of::<C>(), duplicate)) {
            Some(vec) => Some(vec.downcast_ref::<Vec<C::Entry>>().expect(&format!(
                "could not downcast data of {} to a {}",
                type_name::<C>(),
                type_name::<Vec<C::Entry>>(),
            ))),
            None => None,
        }
    }

    pub fn write_to<C: Column>(&mut self, val: C::Entry) {
        self.entries
            .entry((TypeId::of::<C>(), WRITE_SLOT))
            .or_insert(Box::new(Vec::<C::Entry>::new()))
            .downcast_mut::<Vec<C::Entry>>()
            .expect(&format!(
                "could not downcast data of {} to a {}",
                type_name::<C>(),
                type_name::<Vec<C::Entry>>(),
            ))
            .push(val);
        self.action_index.push(ActionEntry::WriteVal {
            into: TypeId::of::<C>(),
        });
    }

    #[must_use]
    pub fn add_ctx_to<C: Column>(&mut self, ctx: C::Entry) -> ContextHandle {
        let duplicate = self
            .available_duplicates
            .entry(TypeId::of::<C>())
            .or_insert_with(|| Slots::<CTX_COUNT_START>::new())
            .allocate();

        self.entries
            .entry((TypeId::of::<C>(), duplicate))
            .or_insert(Box::new(Vec::<C::Entry>::new()))
            .downcast_mut::<Vec<C::Entry>>()
            .expect(&format!(
                "could not downcast data of {} to a {}",
                type_name::<C>(),
                type_name::<Vec<C::Entry>>(),
            ))
            .push(ctx);

        self.action_index.push(ActionEntry::WriteCtx {
            into: TypeId::of::<C>(),
            duplicate,
        });

        ContextHandle {
            contexter: self,
            inserted_to: (TypeId::of::<C>(), duplicate),
        }
    }

    pub fn iter(&self) -> FrameIter<'_> {
        FrameIter {
            contexter: self,
            actions: self.action_index.iter(),
            frame: Frame { context: (), write: (), contexter: () },
            counts: HashMap::new(),
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

#[derive(Debug, Clone)]
pub struct Frame<'contexter> {
    context: HashMap<TypeId, HashMap<ColDuplicate, usize>>,
    write: (TypeId, usize),
    contexter: &'contexter Contexter,
}

#[macro_export]
macro_rules! frame_tuple {
    ($frame:ident => ($value_col:ident) with $($ctx_col:ident),*) => {
        {
            pub fn get_tuple<$($name: Column)*, 'contexter>(frame: &'contexter Frame) -> ($(Option<&$name::Entry>),*){
                (self.get_value::<$value_col>(), ($(self.get_context::<$name>()),*))
            }
            get_tuple($frame)
        }
    };
}

impl<'contexter> Frame<'contexter> {
    pub fn get_context<C: Column>(&self) -> Option<Vec<&C::Entry>> {
        Some(
            self.context
                .get(&TypeId::of::<C>())?
                .iter()
                .map(|(duplicate, index)| {
                    self.contexter
                        .get_column::<C>(*duplicate)
                        .expect(&format!(
                            "missing column {} with duplicate {:?} in frame",
                            type_name::<C>(),
                            duplicate
                        ))
                        .get(*index)
                        .expect(&format!(
                            "missing index {} in column {} with duplicate {:?} in frame",
                            index,
                            type_name::<C>(),
                            duplicate
                        ))
                })
                .collect(),
        )
    }

    pub fn get_value<C: Column>(&self) -> Result<&C::Entry, ()> {
        let (type_id, index) = self.write;
        if type_id != TypeId::of::<C>() {
            return Err(());
        }
        Ok(&self
            .contexter
            .get_column::<C>(WRITE_SLOT)
            .expect(&format!("missing column {} in frame", type_name::<C>()))
            .get(index)
            .expect(&format!(
                "missing index {} in column {} in frame",
                index,
                type_name::<C>()
            )))
    }
}

pub struct FrameIter<'a> {
    contexter: &'a Contexter,
    actions: std::slice::Iter<'a, ActionEntry>,
    frame: Frame<'a>,
    context_counts: HashMap<(TypeId, ColDuplicate), usize>,
    val_counts: HashMap<TypeId, usize>,
}

impl<'a> Iterator for FrameIter<'a> {
    type Item = Frame<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(action) = self.actions.next() {
            match action {
                ActionEntry::WriteCtx { into, duplicate } => {
                    let count = self.context_counts.entry((*into, *duplicate)).or_insert(0);
                    self.frame
                        .context
                        .entry(*into)
                        .or_insert(HashMap::new())
                        .insert(*duplicate, *count);
                    *count += 1;
                }
                ActionEntry::PopCtx { from, duplicate } => {
                    let _context = self
                        .frame
                        .context
                        .get_mut(from)
                        .expect(&format!(
                            "missing context for {} with duplicate {:?} in frame",
                            type_name::<TypeId>(),
                            duplicate
                        ))
                        .remove(duplicate)
                        .expect(&format!(
                            "missing duplicate {:?} in context for {} in frame",
                            duplicate,
                            type_name::<TypeId>()
                        ));
                }
                ActionEntry::WriteVal { into } => {
                    let count = self.val_counts.entry(*into).or_insert(0);
                    self.frame.write = (*into, *count);
                    *count += 1;
                    // Only yield on WriteVal
                    return Some(self.frame.clone());
                }
            }
        }
        None
    }
}
