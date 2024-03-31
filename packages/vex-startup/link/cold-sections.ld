SECTIONS {
	.hot (NOLOAD) : {
		/* __hot_start already defined */
		KEEP(*(.hot_magic))
		. = 0x8;
		__hot_entry_point = .;
		__hot_end = .;
	} > HOT

	.text : {
		__text_start = .;
		KEEP(*(.cold_magic))
        __cold_magic = .;
		. = 0x20;
		*(.boot)
		*(.text .text.* .gnu.linkonce.t.*)
		*(.plt)
		*(.gnu_warning)
		*(.gnu_except_table)
		*(.glue_7 .glue_7t)
		/* -mfloat-abi=softfp */
		*(.vfp11_veneer)
		*(.ARM.extab .gnu.linkonce.armextab.*)
		__text_end = .;
	} > ACTIVE

	.rodata : {
		__rodata_start = .;
		*(.rodata .rodata.* .gnu.linkonce.r.*)
		*(.rodata1 .rodata1.*)
		__rodata_end = .;
	} > ACTIVE

	/* "small" data and BSS; should be next to rodata */
	.sdata : {
		__sdata_start = .;
		*(.sdata .sdata.* .gnu.linkonce.s.*)
		__sdata_end = .;
	} > ACTIVE
	.sdata2 : {
		__sdata2_start = .;
		*(.sdata2 .sdata2.* .gnu.linkonce.s2.*)
		__sdata2_end = .;
	} > ACTIVE

	.data : {
		__data_start = .;
		*(.data .data.* .gnu.linkonce.d.*)
		*(.data1 .data1.*)
		*(.jcr .got .got.plt)
		__data_end = .;
	} > ACTIVE
	.tdata : {
		__tdata_start = .;
		*(.tdata .tdata.* .gnu.linkonce.td.*)
		__tdata_end = .;
	} > ACTIVE

	/* constructors and destructors */
	.init : {
		KEEP (*(.init))
	} > ACTIVE
	.fini : {
		KEEP (*(.fini))
	} > ACTIVE
	.preinit_array : {
		__preinit_array_start = .;
		KEEP (*(SORT_BY_INIT_PRIORITY(.preinit_array.*)))
		KEEP (*(.preinit_array))
		__preinit_array_end = .;
	} > ACTIVE
	.init_array : {
		__init_array_start = .;
		KEEP (*(SORT_BY_INIT_PRIORITY(.init_array.*) SORT_BY_INIT_PRIORITY(.ctors.*)))
		KEEP (*(.init_array .ctors))
		__init_array_end = .;
	} > ACTIVE
	.fini_array : {
		__fini_array_start = .;
		KEEP (*(SORT_BY_INIT_PRIORITY(.fini_array.*) SORT_BY_INIT_PRIORITY(.dtors.*)))
		KEEP (*(.fini_array .dtors))
		__fini_array_end = .;
	} > ACTIVE

	/* what is this? */
	.fixup : {
		__fixup_start = .;
		*(.fixup)
		__fixup_end = .;
	} > ACTIVE

	/* exception handling */
	.eh_frame : {
		__eh_frame_start = .;
		*(.eh_frame)
		__eh_frame_end = .;
	} > ACTIVE
	.eh_framehdr : {
		__eh_framehdr_start = .;
		*(.eh_framehdr)
		__eh_framehdr_end = .;
	} > ACTIVE
	.gcc_except_table : {
		__except_table_start = .;
		*(.gcc_except_table)
		__except_table_end = .;
	} > ACTIVE
	/* stack unwinding */
	.ARM.exidx : {
		__exidx_start = .;
		*(.ARM.exidx* .gnu.linkonce.armexidx.*.*)
		__exidx_end = .;
	} > ACTIVE

	.ARM.attributes : {
		__arm_attributes_start = .;
		*(.ARM.attributes)
		__arm_attributes_end = .;
	} > ACTIVE

	.bss (NOLOAD) : {
		__bss_start__ = .; /* newlib */
		__bss_start = .;
		*(.bss .bss.* .gnu.linkonce.b.*)
		*(COMMON) /* legacy */
		__bss_end = .;
		__bss_end__ = .; /* newlib */
	} > ACTIVE
	.sbss (NOLOAD) : {
		__sbss_start = .;
		*(.sbss2 .sbss2.* .gnu.linkonce.sb.*)
		__sbss_end = .;
	} > ACTIVE
	.sbss2 (NOLOAD) : {
		__sbss2_start = .;
		*(.sbss2 .sbss2.* .gnu.linkonce.sb2.*)
		__sbss2_end = .;
	} > ACTIVE
	.tbss (NOLOAD) : {
		__tbss_start = .;
		*(.tbss .tbss.* .gnu.linkonce.tb.*)
		__tbss_end = .;
	} > ACTIVE

	/* stack and heap */
	.heap (NOLOAD) : ALIGN(4) {
		/* __heap_start already defined */
		. += __heap_length;
		__heap_end = .;
	} > HEAP
	.stack (NOLOAD) : ALIGN(8) { /* EABI: stack must be 8-byte aligned */
		__user_stack_end = .; /* grows down */
		. += __user_stack_length;
		__user_stack_start = .;
		. = ALIGN(8);
		__kernel_stack_end = .;
		. += __kernel_stack_length;
		__kernel_stack_start = .;
		__stack = .; /* newlib */
	} > COLD

	_end = .;
}
