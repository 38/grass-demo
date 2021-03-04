use std::cell::UnsafeCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
use std::{
    borrow::Borrow,
    fmt::{Debug, Formatter, Result as FmtResult},
};

pub trait WithChromSet<H: ChromSetHandle> {
    type Result;
    fn with_chrom_set(self, handle: &mut H) -> Self::Result;
}

pub trait ChromName: Ord + Clone {
    fn to_string(&self) -> String;
    fn write<W: Write>(&self, w: W) -> std::io::Result<()>;
}

impl ChromName for String {
    fn to_string(&self) -> String {
        self.clone()
    }

    fn write<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(self.as_bytes())
    }
}

impl<'a> ChromName for &'a str {
    fn to_string(&self) -> String {
        str::to_string(self)
    }

    fn write<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        w.write_all(self.as_bytes())
    }
}

pub trait ChromSetHandle {
    type RefType: ChromName;
    fn query_or_insert(&mut self, name: &str) -> Self::RefType;
}

pub trait ChromSet {
    type RefType: ChromName;
    type Handle: ChromSetHandle<RefType = Self::RefType>;
    fn get_handle(&self) -> Self::Handle;
}

#[derive(Default)]
struct StringPool {
    s2i_map: HashMap<String, usize>,
    i2s_map: Vec<String>,
}

impl StringPool {
    fn query_id(&self, s: &str) -> Option<usize> {
        if let Some(id) = self.s2i_map.get(s) {
            Some(*id)
        } else {
            None
        }
    }

    fn query_id_or_insert<T: Borrow<str>>(&mut self, s: T) -> usize {
        if let Some(id) = self.query_id(s.borrow()) {
            id
        } else {
            self.s2i_map
                .insert(s.borrow().to_string(), self.i2s_map.len());
            self.i2s_map.push(s.borrow().to_string());
            self.i2s_map.len() - 1
        }
    }
}

type SharedStringPool = Arc<UnsafeCell<StringPool>>;

pub struct LexicalChromSet {
    pool: SharedStringPool,
}

#[derive(Clone)]
pub struct LexicalChromRef {
    pool: SharedStringPool,
    idx: usize,
}

impl Debug for LexicalChromRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = self.to_string();
        write!(f, "{}(Id={})", value, self.idx)
    }
}

pub struct LexicalChromHandle {
    pool: SharedStringPool,
}

impl LexicalChromRef {
    // This is unsafe, since it returns the a reference of the string pool
    // However, our contract is we doesn't allow any long living string pool reference
    // So the only correct way to use this method is DO NOT release this reference.
    unsafe fn get_string_ref(&self) -> &str {
        let pool_ref = &*self.pool.get();
        pool_ref.i2s_map[self.idx].as_ref()
    }
}

impl PartialEq for LexicalChromRef {
    fn eq(&self, other: &Self) -> bool {
        if self.pool.get() == other.pool.get() {
            return self.idx == other.idx;
        }
        unsafe { self.get_string_ref() == other.get_string_ref() }
    }
}

impl PartialOrd for LexicalChromRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.pool.get() == other.pool.get() && self.idx == other.idx {
            return Some(Ordering::Equal);
        }
        unsafe { Some(Ord::cmp(self.get_string_ref(), other.get_string_ref())) }
    }
}

impl Eq for LexicalChromRef {}

impl Ord for LexicalChromRef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl ChromName for LexicalChromRef {
    fn to_string(&self) -> String {
        let ret = unsafe { self.get_string_ref().to_string() };
        ret
    }

    fn write<W: Write>(&self, mut w: W) -> std::io::Result<()> {
        let name = unsafe { self.get_string_ref() };
        w.write_all(name.as_bytes())
    }
}

impl ChromSetHandle for LexicalChromHandle {
    type RefType = LexicalChromRef;
    fn query_or_insert(&mut self, name: &str) -> Self::RefType {
        let pool = unsafe { self.pool.get().as_mut().unwrap() };
        let idx = pool.query_id_or_insert(name);
        LexicalChromRef {
            pool: self.pool.clone(),
            idx,
        }
    }
}

impl LexicalChromSet {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(UnsafeCell::new(StringPool::default())),
        }
    }
}

impl ChromSet for LexicalChromSet {
    type Handle = LexicalChromHandle;
    type RefType = LexicalChromRef;
    fn get_handle(&self) -> Self::Handle {
        LexicalChromHandle {
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_lexical_chrom_set() {
        let chrom_set = LexicalChromSet::new();

        let mut handle_1 = chrom_set.get_handle();
        let mut handle_2 = chrom_set.get_handle();

        let ref1 = handle_1.query_or_insert("chr1");
        let ref2 = handle_2.query_or_insert("chr1");

        assert_eq!(ref1, ref2);
        assert_eq!(ref1.idx, ref2.idx);

        let ref3 = handle_1.query_or_insert("chr2");
        assert_ne!(ref1, ref3);

        assert!(ref3 > ref1);
    }
}
