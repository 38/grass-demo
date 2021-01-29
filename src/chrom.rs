use std::cmp::{PartialOrd, Ordering};
use std::cell::RefCell;

pub trait Chrom : PartialOrd {
    fn name(&self) -> &str;
}

pub struct ChromList {
    list: RefCell<Vec<Box<str>>>,
}

impl <T: AsRef<str> + PartialOrd> Chrom for T {
    fn name(&self) -> &str {
        self.as_ref()
    }
}

#[derive(Clone)]
pub struct ChromListRef<'a> {
    chrom_list: &'a ChromList,
    chrom_id: usize,
}
impl <'a> PartialEq for ChromListRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.chrom_list as *const _ == other.chrom_list as *const _ &&
            self.chrom_id == other.chrom_id
    }
}

impl <'a> PartialOrd for ChromListRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.chrom_list as *const _ != other.chrom_list as *const _ {
            return None;
        }
        self.chrom_id.partial_cmp(&other.chrom_id)
    }
}

impl <'a> Chrom for ChromListRef<'a> {
    fn name(&self) -> &str {
        let list = self.chrom_list.list.borrow();
        // Justify: This is safe, as long as the ChromListRef cannot live longer than its backed
        // ChromList, thus the boxed strs won't get dropped, though the list might be resized
        unsafe {
            let data:&str = &list.get_unchecked(self.chrom_id);
            std::mem::transmute(data)
        }
    }
}

impl ChromList {
    pub fn new() -> Self {
        Self {
            list: RefCell::new(Vec::new())
        }
    }

    pub fn query<T: Into<String> + AsRef<str>>(&self, name: T) -> ChromListRef {
        let list = self.list.borrow();
        let mut idx = None;
        if let Some(value) = list.last() {
            if value.as_ref() == name.as_ref() {
                idx = Some(list.len() - 1);
            }
        }
        idx = idx.or_else(|| list.iter().enumerate().find_map(|(idx,what)| if what.as_ref() == name.as_ref() {
                Some(idx)
            } else {
                None
            }
        ));
        drop(list);

        if idx.is_none() {
            let mut list = self.list.borrow_mut();
            idx = Some(list.len());
            let data = name.into().into_boxed_str();
            list.push(data);
        }

        ChromListRef {
            chrom_list: self,
            chrom_id: idx.unwrap(),
        }
    }
}
