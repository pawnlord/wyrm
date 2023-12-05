from enum import Enum

contents = ""
with open("out.dump") as f:
    contents = f.read()

functions = "\n".join(contents.split("\n; function ")[1:-2])
lines = functions.split('\n')

ops = {}

class Type(Enum):
    VOID = (0, "void")
    I32 = (1, "i32")
    I64 = (2, "i64")
    F32 = (3, "f32")
    F64 = (4, "f64")

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
        if information["value"] in ops:
            ops[information["value"]] = (information["name"], ops[information["value"]][1], information["types"])
        else:
            ops[information["value"]] = (information["name"], False, information["types"])
    else:
        success, prev_information = get_line_values(lines[i-1])
        if not success:
            continue

        # It can't not be, we just went over that line in the previous iteration
        # If it isn't, then we are probably in a br_table
        if prev_information["value"] in ops:
            ops[prev_information["value"]] = (ops[prev_information["value"]][0], True, ops[prev_information["value"]][2])
        

items = ops.items()
items = sorted(items, key=lambda x: x[0])
for key, value in items:
    if value[0] == None:
        value = ("None", value[1])
    print(f"op: {key:<30} name: {value[0]:<30} args: {value[1]:<30} result_type: {value[2][0]:<32} arg_type: {value[2][1]:<32}")