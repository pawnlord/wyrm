import pickle

from shared import *

import subprocess

import os
 
wabt_path = "../wabt/bin/wat2wasm"

lines = []
def get_lines_file(f):
    global lines

    if os.path.isfile(f +".cache"):
        with open(f + ".cache") as file:
            lines.extend(file.readlines())
            return

    proc = subprocess.Popen([wabt_path, f, "-v", "--output=a.wasm"], stderr=subprocess.PIPE)
    contents = proc.stderr.read().decode()
    functions = "\n".join(contents.split("\n; function ")[1:-2])
    lines += functions.split('\n')

    with open(f + ".cache", "w") as file:
        file.write("\n".join(lines)) 

def get_lines(dir):
    for filename in os.listdir(dir):
        f = os.path.join(dir, filename)
        if os.path.isdir(f):
            get_lines(f)
        elif os.path.isfile(f) and filename.endswith(".wat"):
            # print(f)
            get_lines_file(f)

# get_lines("binaryen_tests")
get_lines_file("earthplugin_web.wat")
get_lines_file("snake.wat")


lines.append("0001973: d2                                        ; ref.func")
lines.append("0001973: 00                                        ; function index")

text = "Scraped content from WASM"
print("*" + ("="*20) + f" {text:<20} " + ("="*20) + "*")

ops = {}

already_parsed = {}
def get_line_values(i, line):
    if i in already_parsed.keys():
        return already_parsed[i]
    left_split = line.split(':')
    if len(left_split) == 2:
        right_split = left_split[1].split(';')
        if len(right_split) != 2:
            return (False, None)
        value = right_split[0].strip()
        name = right_split[1].strip()
        parts = name.split('.')

        types = (get_type(parts[0]), Type.VOID)
        if len(parts) == 2 and not ("load" in parts[1] or "store" in parts[1]):
            types = (types[0], get_type(parts[1]))
        
        isInstr = not (" " in name or " " in value)
        isInstr &= name != "alignment"
        isInstr &= not name in ["i32", "i64", "f32", "f64"]

        values = (True,
                {
                    "value": value,
                    "name": name,
                    "isInstr": isInstr,
                    "isAlign": name == "alignment",
                    "types": types
                })
        already_parsed[i] = values
        return values
    return (False, None)
    

for i, line in enumerate(lines):
    if i % 100 == 0:
        print(i)
    success, information = get_line_values(i, line)

    if not success:
        continue

    if information["isInstr"]:
        ops[information["value"]] = OpInfo(information["name"], False, False, information["types"])
    else:
        add_alignment = False

        success, prev_information = get_line_values(i-1, lines[i-1])
        if not success:
            continue

        temp = i - 1
        while prev_information["isAlign"]:
            temp -= 1
            add_alignment = True
            success, prev_information = get_line_values(temp, lines[temp])
            if not success:
                break
        
        if not success:
            continue
        # It can't not be, we just went over that line in the previous iteration
        # If it isn't, then we are probably in a br_table
        if prev_information["value"] in ops:
            prev_op = ops[prev_information["value"]]
            ops[prev_information["value"]] = OpInfo (
                prev_op.name, 
                True, 
                prev_op.takes_alignment or add_alignment, 
                prev_op.type
            )
ops["call"].type[0] = Type.FUNC
# TODO: Check that this is not picking up of func second
ops["ref.func"].type[0] = Type.FUNC

text = "Collected instructions in output"
print("*" + ("="*20) + f" {text:<30} " + ("="*20) + "*")
items = ops.items()
items = sorted(items, key=lambda x: x[0])
for key, value in items:
    if value.name == None:
        value = ("None", value.has_arg)
    print(f"op: {key:<30} name: {value.name:<30} args: {value.has_arg:<30} result_type: {value.type[0]:<32} arg_type: {value.type[1]:<32} aligned: {value.takes_alignment:<32}")

with open("ops.pkl", "wb") as f:
    pickle.dump(ops, f)