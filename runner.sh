echo "stripping..."
arm-none-eabi-objcopy --strip-symbol=install_hot_table --strip-symbol=__libc_init_array --strip-symbol=_PROS_COMPILE_DIRECTORY --strip-symbol=_PROS_COMPILE_TIMESTAMP --strip-symbol=_PROS_COMPILE_TIMESTAMP_INT $1 $1.stripped
echo "[ok]"
echo "converting elf to bin..."
arm-none-eabi-objcopy -O binary -R .hot_init $1.stripped  $1.bin
echo "[ok]"
echo "output at $1.bin"

pros upload --target v5 -af screen --slot 1 $1.bin