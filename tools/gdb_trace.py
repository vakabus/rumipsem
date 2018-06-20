import gdb
import json


def register_value(name):
    res = gdb.execute('i r {}'.format(name), to_string=True).strip()
    return int(res.split(" ")[1], 16)

def dump_registers():
    mapping = {
        1: "at",
        2: "v0",
        3: "v1",
        4: "a0",
        5: "a1",
        6: "a2",
        7: "a3",
        8: "t0",
        9: "t1",
        10: "t2",
        11: "t3",
        12: "t4",
        13: "t5",
        14: "t6",
        15: "t7",
        16: "s0",
        17: "s1",
        18: "s2",
        19: "s3",
        20: "s4",
        21: "s5",
        22: "s6",
        23: "s7",
        24: "t8",
        25: "t9",
        26: "k0",
        27: "k1",
        28: "gp",
        29: "sp",
       # 30: "fp",
        31: "ra",
    }
    registers = {k: register_value(v) for k, v in mapping.items()}
    return registers


outfile = open("trace", "w")
while True:
    instructions = gdb.execute('x /i $pc', to_string=True).rstrip('\n').split('\n')
    for i,inst in enumerate(instructions):
        pc = int(inst[3:].split(':')[0],16)
        record = {
            "address": pc,
            "registers": {}
        }
        if i == len(instructions) - 1:
            record["registers"] = dump_registers()

        print(json.dumps(record), file=outfile)

    gdb.execute('stepi', to_string=False)
    gdb.flush()
outfile.close()
