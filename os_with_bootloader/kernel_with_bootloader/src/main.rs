#![no_std]
#![no_main]
#![feature(allow_internal_unstable)]
#[macro_use]
mod std;
//demanded by #[allow_internal_unstable(print_internals, format_args_nl)] in my std.rs
//below is for x86 interrupts
#![feature(abi_x86_interrupt)]
mod interrupts;
mod smart_pointer_examples;
pub(crate) mod std;
pub mod task;
mod task_example;
mod writer;

use alloc::{borrow::ToOwned, sync::Arc};
//use bootloader_api::config::Mapping;
use writer::FrameBufferWriter;
use x86_64::instructions::hlt;

//let's get heap memory allocation going
extern crate alloc;
use good_memory_allocator::SpinLockedAllocator;

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

use bootloader_api::{
    config::Mapping,
    info::{MemoryRegion, MemoryRegionKind},
};

//Use the entry_point macro to register the entry point function: bootloader_api::entry_point!(kernel_main)
//optionally pass a custom config
pub static BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config.kernel_stack_size = 100 * 1024; // 100 KiB
    config
};
bootloader_api::entry_point!(my_entry_point, config = &BOOTLOADER_CONFIG);

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    std::input_str,
    task::{simple_executor::SimpleExecutor, Task},
};
//use lazy static to allow declaration of static without initializing with a constant value
//Mutex from spin is used for control of threads access.
lazy_static! {
    pub(crate) static ref FRAME_BUFFER_WRITER: Mutex<FrameBufferWriter> =
        Mutex::new(FrameBufferWriter::empty());
}

fn my_entry_point(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let frame_buffer_info = boot_info.framebuffer.as_mut().unwrap().info();

    let buffer = boot_info.framebuffer.as_mut().unwrap().buffer_mut();

    // let mut frame_buffer_writer = FrameBufferWriter::new(buffer, frame_buffer_info);
    FRAME_BUFFER_WRITER.lock().init(buffer, frame_buffer_info);
    FRAME_BUFFER_WRITER.lock().set_x_y_pos(None, Some(100));
    // use core::fmt::Write; //below requires this
    // writeln!(
    //     frame_buffer_writer,
    //     "Testing testing {} and {}",
    //     1,
    //     4.0 / 2.0
    // )
    // .unwrap();
    // frame_buffer_writer.set_posi(658, 48);
    // writeln!(frame_buffer_writer, "Hello ^_^");
    // frame_buffer_writer.set_posi(1, 20);
    // writeln!(frame_buffer_writer, "Testing blank screen 2");
    // macro_rules! println {
    //     ($($a:tt)*) => {{
    //         writeln!(frame_buffer_writer, $($a)*).unwrap();
    //     }};
    // }

    // // ...

    // // Replace the standard println! calls with your custom macro.
    // println!("Hi This Is Print");
    // println!("Hi This is Also Print");

    //let's initialize our global memory allocator
    let last_memory_region = boot_info.memory_regions.last().unwrap();

    //get the first bootload memory
    let mut boot_loader_memory_region = MemoryRegion::empty();

    for memory_region in boot_info.memory_regions.iter() {
        match memory_region.kind {
            MemoryRegionKind::Bootloader => {
                boot_loader_memory_region = *memory_region;
                break;
            }
            _ => continue,
        }
    }
    let physical_memory_offset = boot_info.physical_memory_offset.into_option().unwrap();

    let heap_start = boot_loader_memory_region.end + 0x1 + physical_memory_offset;
    let heap_size = last_memory_region.end - (boot_loader_memory_region.end + 0x1);

    unsafe {
        ALLOCATOR.init(heap_start as usize, heap_size as usize);
    }
    use alloc::boxed::Box;

    let x: Box<i32> = Box::new(33);
    println!("\nValue in heap is {}", &x);

    let y: i32 = 33;
    println!("\nValue in stack is {}", &y);
    //Let's see some more smart pointer examples
    use smart_pointer_examples::*;
    box_vs_rc();

    let root = create_tree();
    add_child(&root);
    print_tree(root);
    interrupts::init();

    //Let's experience getting string from keyboard and saving into a variable for use
    print!("Enter string: ");
    let input = match input_str() {
        Some(value) => value,
        None => "".to_owned(),
    };
    println!("\nString entered is '{}'", input);
    let returned_input = input_str!("This Is My input Macro Testing");
    println!("\nYour string is '{}'", returned_input);


    // invoke a breakpoint exception for test
    //x86_64::instructions::interrupts::int3();

    //println!("Did not crash after breakpoint exception");

    // Below can trigger a page fault. Just for test
    /*
    unsafe {
        *(0xdeadbeef as *mut u8) = 42; //invalid memory address
    };*/

    loop {
        hlt(); //stop x86_64 from being unnecessarily busy while looping
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        hlt();
    }
}
