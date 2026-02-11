use crate::process_mem::{f_read, f_write};

pub struct Addresses {
    addresses: [usize; 4],
    saved_values: [f32; 4],
}

impl Addresses {
    pub fn new(addresses: [usize; 4]) -> Self {
        Self { addresses, saved_values: Default::default() }
    }

    pub fn populate_save(&mut self) {
        for (save, addr) in
            self.saved_values.iter_mut().zip(self.addresses.iter())
        {
            *save = f_read(*addr);
        }
    }

    pub fn sum(&mut self, value: f32) {
        for (save, addr) in
            self.saved_values.iter_mut().zip(self.addresses.iter())
        {
            *save += value;
            f_write(*addr, *save);
        }
    }

    pub fn keep(&mut self) {
        for (save, addr) in
            self.saved_values.iter_mut().zip(self.addresses.iter())
        {
            f_write(*addr, *save);
        }
    }
}
