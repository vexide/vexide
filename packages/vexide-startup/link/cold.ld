INCLUDE prelude.ld
INCLUDE memory.ld

ENTRY(_entry)

REGION_ALIAS("ACTIVE", COLD)
INCLUDE cold-sections.ld
