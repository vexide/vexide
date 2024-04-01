INCLUDE prelude.ld
INCLUDE memory.ld

ENTRY(_entry)

REGION_ALIAS("ACTIVE", COLD)
INCLUDE cold-sections.ld

__sysapi_func_table_start = 0x037fc000;
