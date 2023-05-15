import subprocess
import sys

output_path = sys.argv[1]

elf_to_bin_cmd = "arm-none-eabi-objcopy -O binary -R .hot_init " + output_path + " " + output_path + ".bin"
subprocess.run(elf_to_bin_cmd, shell=True)

print("uploading " + output_path + ".bin")
upload_cmd = "pros upload --target v5 --slot 1 " + output_path + ".bin"
subprocess.run(upload_cmd, shell=True)