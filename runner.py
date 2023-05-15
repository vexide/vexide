import subprocess
import sys

output_path = sys.argv[1]
if sys.argv.__len__() > 2 and sys.argv[2] == "-v":
    verbose = True
else:
    verbose = False

strip_cmd = "arm-none-eabi-objcopy --strip-symbol=install_hot_table --strip-symbol=__libc_init_array --strip-symbol=_PROS_COMPILE_DIRECTORY --strip-symbol=_PROS_COMPILE_TIMESTAMP --strip-symbol=_PROS_COMPILE_TIMESTAMP_INT " + output_path + " " + output_path + ".stripped"
elf_to_bin_cmd = "arm-none-eabi-objcopy -O binary -R .hot_init " + output_path + ".stripped " + output_path + ".bin"

print("stripping binary.")
if verbose:
    print(strip_cmd)

subprocess.run(strip_cmd, shell=True)

print("creating raw binary.")
if verbose:
    print(elf_to_bin_cmd)

subprocess.run(elf_to_bin_cmd, shell=True)

print("uploading " + output_path + ".bin")
upload_cmd = "pros upload --target v5 --slot 1 " + output_path + ".bin"
subprocess.run(upload_cmd, shell=True)