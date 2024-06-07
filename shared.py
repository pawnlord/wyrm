from enum import Enum

class OpInfo:
    def __init__(self, name, has_arg, takes_alignment, type):
        self.name = name
        self.has_arg = has_arg
        self.takes_alignment = takes_alignment
        self.type = type

class Type(Enum):
    VOID = (0, "void", "Void")
    I32 = (1, "i32", "I32")
    I64 = (2, "i64", "I64")
    F32 = (3, "f32", "F32")
    F64 = (4, "f64", "F64")
    LOCAL = (4, "local", "Global")
    GLOBAL = (4, "global", "Local")
    FUNC = (4, "func", "Func")

def get_type(s):
    for t in Type:
        if t.value[1] in s:
            return t
    return Type.VOID
