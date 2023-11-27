use std::{
    borrow::Cow,
    cell::RefCell,
    collections::{hash_map::Entry, HashMap},
    fs,
    path::Path,
};

use anyhow::Result;

use crate::bootstrap::{FUNC_EXEC, PREFIX, PROGRAM_COUNTER};

pub struct Block<'a> {
    id: u64,
    content: RefCell<String>,
    fn_name: Cow<'a, str>,
}

impl Block<'_> {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn push_str(&self, s: impl AsRef<str>) {
        self.content.borrow_mut().push_str(s.as_ref())
    }

    pub fn fn_name(&self) -> &str {
        &self.fn_name
    }
}

pub(super) struct Context<'a> {
    mangle_uuid: u64,
    anonymous_pool: u64,
    label_id_pool: u64,
    labels: HashMap<Cow<'a, str>, Block<'a>>,
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            mangle_uuid: rand::random(),
            anonymous_pool: 0,
            label_id_pool: 0, // 0 for empty block
            labels: HashMap::new(),
        }
    }

    fn gen_block(&mut self, fn_name: Cow<'a, str>) -> Block<'a> {
        let new_id = self
            .label_id_pool
            .checked_add(1)
            .expect("label id pool was full");
        self.label_id_pool = new_id;

        Block {
            id: new_id,
            content: Default::default(),
            fn_name,
        }
    }

    fn mangle(&self, label: &str) -> String {
        format!("{PREFIX}_{label}_mangled_{:x}", self.mangle_uuid)
    }

    pub fn insert_label(&mut self, key: &'a str, mangle: bool) -> &mut Block<'a> {
        let block: Block<'a> = self.gen_block(if mangle {
            self.mangle(key).into()
        } else {
            key.into()
        });

        match self.labels.entry(key.into()) {
            Entry::Occupied(_) => panic!("label is exists"),
            Entry::Vacant(vac) => vac.insert(block),
        }
    }

    pub fn get_label(&self, key: &str) -> &Block<'a> {
        self.labels.get(key).expect("label not defined")
    }

    pub fn new_anonymous_label(&mut self) -> String {
        let id = self.anonymous_pool;
        self.anonymous_pool += 1;

        let label = format!("_anonymous_{id:x}");
        let block = self.gen_block(self.mangle(&label).into());
        self.labels.insert(label.clone().into(), block);
        label
    }

    pub fn generate(&self, save_as: impl AsRef<Path>) -> Result<()> {
        let mut id_table: Vec<&Block> = self.labels.values().collect();
        id_table.sort_by_key(|func| func.id);

        crate::bootstrap::gen_bin_search(
            save_as.as_ref(),
            FUNC_EXEC,
            PROGRAM_COUNTER,
            self.labels.len() + 1,
            |nth| match nth.checked_sub(1) {
                None => format!("function {PREFIX}_nonexistence_fn"),
                Some(nth2) => self
                    .labels
                    .get(id_table[nth2].fn_name())
                    .unwrap()
                    .fn_name()
                    .to_string(),
            },
        )?;

        for block in self.labels.values() {
            let mut path = save_as.as_ref().join(&*block.fn_name);
            path.set_extension("mcfunction");
            fs::write(path, &*block.content.borrow())?;
        }
        Ok(())
    }
}
