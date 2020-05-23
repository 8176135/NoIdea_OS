use x86_64::VirtAddr; // Only use 48 bits of 64 bits word
use x86_64::structures::{tss::TaskStateSegment, gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector}};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const DF_STACK_SIZE: usize = 4096;
// Make sure to use mut, so that the data is not mapped to read only storage
static mut DF_STACK: [u8; DF_STACK_SIZE] = [0; DF_STACK_SIZE]; // No stack guard for Double Fault

struct Selectors {
	code_selector: SegmentSelector,
	tss_selector: SegmentSelector,
}

lazy_static! {
	static ref TSS: TaskStateSegment = create_task_state_segment();
	static ref GDT: (GlobalDescriptorTable, Selectors) = create_gdt();
}

pub fn gdt_init() {
	GDT.0.load();
	unsafe {
		x86_64::instructions::segmentation::set_cs(GDT.1.code_selector);
		x86_64::instructions::tables::load_tss(GDT.1.tss_selector);
	}
}

fn create_task_state_segment() -> TaskStateSegment {
	let mut tss = TaskStateSegment::new();
	tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
		let df_stack_start = VirtAddr::from_ptr(unsafe { &DF_STACK });
		let df_stack_end = df_stack_start + DF_STACK_SIZE;
		df_stack_end // Stack grows downwards
	};
	tss
}

fn create_gdt() -> (GlobalDescriptorTable, Selectors) {
	let mut gdt = GlobalDescriptorTable::new();
	let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
	let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
	(gdt, Selectors { code_selector, tss_selector })
}
