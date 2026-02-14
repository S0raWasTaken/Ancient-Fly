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
        let first_value = unsafe { f_read(self.addresses[0]) };
        self.saved_values[0] = first_value;
        self.saved_values[1] = first_value;
        self.saved_values[2] = unsafe { f_read(self.addresses[2]) };
        self.saved_values[3] = unsafe { f_read(self.addresses[3]) };
    }

    pub fn sum(&mut self, value: f32) {
        self.saved_values[0] += value;
        self.saved_values[1] = self.saved_values[0];
        self.saved_values[2] += value;
        self.saved_values[3] += value;

        self.keep();
    }

    pub fn keep(&mut self) {
        for (save, addr) in self.saved_values.iter().zip(self.addresses.iter())
        {
            unsafe { f_write(*addr, *save) };
        }
    }
}
