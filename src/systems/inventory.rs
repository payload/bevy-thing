#[derive(Default, Debug)]
pub struct Inventory {
    pub items: Vec<&'static str>,
}

impl Inventory {
    pub fn has(&self, item: &'static str) -> bool {
        self.pos(item).is_some()
    }

    pub fn take(&mut self, item: &'static str) {
        for pos in self.pos(item) {
            self.items.remove(pos);
        }
    }

    pub fn put(&mut self, item: &'static str) {
        self.items.push(item);
    }

    pub fn count(&self, item: &'static str) -> usize {
        self.items.iter().filter(|it| **it == item).count()
    }

    fn pos(&self, item: &'static str) -> Option<usize> {
        self.items.iter().position(|it| *it == item)
    }
}
