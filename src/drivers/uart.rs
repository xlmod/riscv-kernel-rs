use core::fmt::{Error, Write};

pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub fn new(base_address: usize) -> Self {
        Self { base_address }
    }

    pub fn init(base_address: usize) {
        let ptr = base_address as *mut u8;
        unsafe {
            // Set the word lenght to 8bit lcr[1:0]
            ptr.add(3).write_volatile((1 << 0) | (1 << 1));

            // Enable FIFO
            ptr.add(2).write_volatile(1 << 0);

            // Enable receiver buffer interrupts
            ptr.add(1).write_volatile(1 << 0);

            // The formula given in the NS16500A specification for calculating the divisor
            // is:
            // divisor = ceil( (clock_hz) / (baud_sps x 16) )
            // So, we substitute our values and get:
            // divisor = ceil( 22_729_000 / (2400 x 16) )
            // divisor = ceil( 22_729_000 / 38_400 )
            // divisor = ceil( 591.901 ) = 592
            let divisor: u16 = 592;
            let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
            let divisor_most: u8 = (divisor >> 8).try_into().unwrap();
            // Notice that the divisor register DLL (divisor latch least) and DLM (divisor
            // latch most) have the same base address as the receiver/transmitter and the
            // interrupt enable register. To change what the base address points to, we
            // open the "divisor latch" by writing 1 into the Divisor Latch Access Bit
            // (DLAB), which is bit index 7 of the Line Control Register (LCR) which
            // is at base_address + 3.
            let lcr = ptr.add(3).read_volatile();
            ptr.add(3).write_volatile(lcr | 1 << 7);
            ptr.add(0).write_volatile(divisor_least);
            ptr.add(1).write_volatile(divisor_most);
            ptr.add(3).write_volatile(lcr);
        }
    }

    pub fn put(&self, c: u8) {
        let ptr = self.base_address as *mut u8;
        unsafe {
            ptr.add(0).write_volatile(c);
        }
    }

    pub fn get(&self) -> Option<u8> {
        let ptr = self.base_address as *mut u8;
        unsafe {
            // Bit index #5 is the Line Control Register
            if ptr.add(5).read_volatile() & 1 == 0 {
                None
            } else {
                Some(ptr.add(0).read_volatile())
            }
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}
