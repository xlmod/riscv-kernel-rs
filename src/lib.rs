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
        use crate::drivers::uart::Uart;
        use crate::MMIO_UART_ADDR;
        let _ = write!(Uart::new(MMIO_UART_ADDR), $($args)+);
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
    use drivers::uart::Uart;
    Uart::init(MMIO_UART_ADDR);
    let serial = Uart::new(MMIO_UART_ADDR);
    println!("HEAP_START: {}", unsafe { HEAP_START });
    println!("HEAP_SIZE: {:x}", unsafe { HEAP_SIZE });
    println!("TEXT_START: {}", unsafe { TEXT_START });
    println!("TEXT_END: {}", unsafe { TEXT_END });
    println!("DATA_START: {}", unsafe { DATA_START });
    println!("DATA_END: {}", unsafe { DATA_END });
    println!("RODATA_START: {}", unsafe { RODATA_START });
    println!("RODATA_END: {}", unsafe { RODATA_END });
    println!("BSS_START: {}", unsafe { BSS_START });
    println!("BSS_END: {}", unsafe { BSS_END });
    println!("KERNEL_STACK_START: {}", unsafe { KERNEL_STACK_START });
    println!("KERNEL_STACK_END: {}", unsafe { KERNEL_STACK_END });
    println!("KERNEL_TABLE: {}", unsafe { KERNEL_TABLE });

    use memory::{
        physical::{
            PhysFrame,
            PhysAddr,
        },
        page::PageType
    };
    unsafe {
        
        let mut phys_alloc = memory::physical::PhysFrameAllocator::new(
            TEXT_START,
            HEAP_START.get_ptr(),
            (MEMORY_END - MEMORY_START).get_u64() as usize);


        let mut tabframe: [PhysFrame; 8] = [PhysFrame::new(PhysAddr::new(0), PageType::Page); 8];

        for i in 0..8 {
            match phys_alloc.alloc(memory::page::PageType::MegaPage) {
                Some(ppf) => {
                    println!("{}", ppf);
                    tabframe[i] = ppf;
                },
                None => println!("Error"),
            }
        }

        match phys_alloc.alloc(memory::page::PageType::Page) {
            Some(ppf) => println!("{}", ppf),
            None => println!("Error"),
        }
        match phys_alloc.alloc(memory::page::PageType::MegaPage) {
            Some(ppf) => println!("{}", ppf),
            None => println!("Error"),
        }
        match phys_alloc.alloc(memory::page::PageType::Page) {
            Some(ppf) => println!("{}", ppf),
            None => println!("Error"),
        }
        match phys_alloc.alloc(memory::page::PageType::MegaPage) {
            Some(ppf) => println!("{}", ppf),
            None => println!("Error"),
        }
    println!("End5");


    }
    println!("End!!");
    //loop {
    //    if let Some(c) = serial.get() {
    //        match c {
    //            127 => {
    //                print!("{} {}", 8 as char, 8 as char);
    //            }
    //            10 | 13 => {
    //                println!();
    //            }
    //            _ => {
    //                print!("{}", c as char);
    //            }
    //        }
    //    }
    //}
}

// ///////////////////////////////////
// / RUST MODULES
// ///////////////////////////////////
pub mod drivers;
pub mod memory;
