"""
A script to read through a dump of wasm code and find what instructions map to what symbol
"""

from enum import Enum
import subprocess

import os
 
wabt_path = "../wabt/bin/wat2wasm"

lines = []
def get_lines_file(f):
    global lines
    proc = subprocess.Popen([wabt_path, f, "-v", "--output=a.wasm"], stderr=subprocess.PIPE)
    contents = proc.stderr.read().decode()
    functions = "\n".join(contents.split("\n; function ")[1:-2])
    lines += functions.split('\n')

def get_lines(dir):
    for filename in os.listdir(dir):
        f = os.path.join(dir, filename)
        if os.path.isdir(f):
            get_lines(f)
        elif os.path.isfile(f) and filename.endswith(".wat"):
            # print(f)
            get_lines_file(f)

get_lines("binaryen_tests")
get_lines_file("earthplugin_web.wat")
get_lines_file("snake.wat")


text = "Scraped content from WASM"
print("*" + ("="*20) + f" {text:<20} " + ("="*20) + "*")

ops = {}

class Type(Enum):
    VOID = (0, "void", "Void")
    I32 = (1, "i32", "I32")
    I64 = (2, "i64", "I64")
    F32 = (3, "f32", "F32")
    F64 = (4, "f64", "F64")

def get_type(s):
    for t in Type:
        if t.value[1] in s:
            return t
    return Type.VOID


def get_line_values(line):
    left_split = line.split(':')
    if len(left_split) == 2:
        right_split = left_split[1].split(';')
        if len(right_split) != 2:
            return (False, None)
        value = right_split[0].strip()
        name = right_split[1].strip()
        parts = name.split('.')

        types = (get_type(parts[0]), Type.VOID)
        if len(parts) == 2:
            types = (types[0], get_type(parts[1]))

        return (True,
                {
                    "value": value,
                    "name": name,
                    "isInstr": not (" " in name or " " in value),
                    "types": types
                })
    return (False, None)
    


for i, line in enumerate(lines):
    success, information = get_line_values(line)

    if not success:
        continue

    if information["isInstr"]:
        ops[information["value"]] = (information["name"], False, information["types"])
    else:
        success, prev_information = get_line_values(lines[i-1])
        if not success:
            continue

        # It can't not be, we just went over that line in the previous iteration
        # If it isn't, then we are probably in a br_table
        if prev_information["value"] in ops:
            ops[prev_information["value"]] = (ops[prev_information["value"]][0], True, ops[prev_information["value"]][2])
        

text = "Collected instructions in output"
print("*" + ("="*20) + f" {text:<30} " + ("="*20) + "*")
items = ops.items()
items = sorted(items, key=lambda x: x[0])
for key, value in items:
    if value[0] == None:
        value = ("None", value[1])
    print(f"op: {key:<30} name: {value[0]:<30} args: {value[1]:<30} result_type: {value[2][0]:<32} arg_type: {value[2][1]:<32}")

# Produce rust representation
print(ops)

text = "Outputting Rust array"
print("*" + ("="*20) + f" {text:<20} " + ("="*20) + "*")

ops_int = {int(k,16): info for k, info in ops.items()}

print('const INSTRS: [InstrInfo; 255] = [')
for i in range(0, 256):
    if i in ops_int.keys():
        item = ops_int[i]
        print('    InstrInfo{instr: 0x%02x, name: "%s", in_type: Type::%s, out_type: Type::%s},' % (i, item[0], item[2][1].value[2], item[2][0].value[2]))
    else:
        print('    InstrInfo{instr: 0x%02x, name: "", in_type: Type::Void, out_type: Type::Void},' % i)
print('];')