MEMORY {
	COLD : ORIGIN = __cold_start, LENGTH = __cold_length
	HEAP : ORIGIN = __heap_start, LENGTH = __heap_length
	HOT : ORIGIN = __hot_start, LENGTH = __hot_length
}
