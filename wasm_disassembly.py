"""
A script to read through a dump of wasm code and find what instructions map to what symbol
"""

import pickle
from shared import *

ops = {}
with open("ops.pkl", "rb") as f:
    ops = pickle.load(f)

text = "Collected instructions in output"
print("*" + ("="*20) + f" {text:<30} " + ("="*20) + "*")
items = ops.items()
items = sorted(items, key=lambda x: x[0])
for key, value in items:
    if value.name == None:
        value = ("None", value.has_arg)
    print(f"op: {key:<30} name: {value.name:<30} args: {value.has_arg:<30} result_type: {value.type[0]:<32} arg_type: {value.type[1]:<32} aligned: {value.takes_alignment:<32}")

# Produce rust representation
print(ops)

text = "Outputting Rust array"
print("*" + ("="*20) + f" {text:<20} " + ("="*20) + "*")

ops_int = {int(k,16): info for k, info in ops.items()}

print('const INSTRS: [InstrInfo; 255] = [')
for i in range(0, 256):
    if i in ops_int.keys():
        item = ops_int[i]
        print('    InstrInfo{instr: 0x%02x, name: "%s", in_type: Type::%s, out_type: Type::%s, has_arg: %s},' % (i, item.name, item.type[1].value[2], item.type[0].value[2], "true" if item.has_arg else "false"))
    else:
        print('    InstrInfo{instr: 0x%02x, name: "", in_type: Type::Void, out_type: Type::Void, has_arg: false},' % i)
print('];')