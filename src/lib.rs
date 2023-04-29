#![no_std]
#![feature(panic_info_message)]

// ///////////////////////////////////
// / RUST MACROS
// ///////////////////////////////////
#[macro_export]
macro_rules! print
{
	($($args:tt)+) => ({
        use core::fmt::Write;
        use $crate::drivers::uart::UART;
        let _ = write!(UART.lock(), $($args)+);
	});
}
#[macro_export]
macro_rules! println
{
	() => ({
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

// ///////////////////////////////////
// / LANGUAGE STRUCTURES / FUNCTIONS
// ///////////////////////////////////
#[no_mangle]
extern "C" fn eh_personality() {}
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    if let Some(p) = info.location() {
        println!(
            "line {}, file {}: {}",
            p.line(),
            p.file(),
            info.message().unwrap()
        );
    } else {
        println!("no information available.");
    }
    abort();
}
#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            core::arch::asm!("wfi");
        }
    }
}

// ///////////////////////////////////
// / CONSTANTS
// ///////////////////////////////////
const MMIO_UART_ADDR: usize = 0x1000_0000;

extern "C" {
    static HEAP_START: memory::physical::PhysAddr;
    static HEAP_SIZE: usize;
    static MEMORY_START: memory::physical::PhysAddr;
    static MEMORY_END: memory::physical::PhysAddr;
    static TEXT_START: memory::physical::PhysAddr;
    static TEXT_END: memory::physical::PhysAddr;
    static DATA_START: memory::physical::PhysAddr;
    static DATA_END: memory::physical::PhysAddr;
    static RODATA_START: memory::physical::PhysAddr;
    static RODATA_END: memory::physical::PhysAddr;
    static BSS_START: memory::physical::PhysAddr;
    static BSS_END: memory::physical::PhysAddr;
    static KERNEL_STACK_START: memory::physical::PhysAddr;
    static KERNEL_STACK_END: memory::physical::PhysAddr;
    static KERNEL_TABLE: memory::physical::PhysAddr;
}

// ///////////////////////////////////
// / ENTRY POINT
// ///////////////////////////////////
#[no_mangle]
extern "C" fn kmain() {
    println!("############################### START ###############################");
    use memory::physical::PHYSFRAMEALLOCATOR;

    unsafe {
        match PHYSFRAMEALLOCATOR.lock().alloc(memory::page::PageType::Page, 8096) {
            Ok(pf) => {
                println!("{}", pf);
            }
            Err(err) => println!("{}", err),
        }
        match PHYSFRAMEALLOCATOR.lock().alloc(memory::page::PageType::MegaPage, 8) {
            Ok(pf) => {
                println!("{}", pf);
            }
            Err(err) => println!("{}", err),
        }
    }

    println!("################################ END ################################");
}

// ///////////////////////////////////
// / RUST MODULES
// ///////////////////////////////////
pub mod drivers;
pub mod memory;
