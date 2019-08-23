from . import ffi
from ctypes import *

data_1 = {
    "S5": {
        "2019-01-01 00:00:00": {
            "Close<EUR_USD>": 1.0
        },
        "2019-01-01 00:00:05": {
            "Close<EUR_USD>": 2.0
        },
        "2019-01-01 00:00:10": {
            "Close<EUR_USD>": 3.0
        },
        "2019-01-01 00:00:15": {
            "Close<EUR_USD>": 4.0
        },
        "2019-01-01 00:00:20": {
            "Close<EUR_USD>": 5.0
        },
    },
}

# class Vec:
#     def __init__(self, vec):
#         self.vec = vec

#     def value(self, i):
#         return self.vec[i]



def test_2():
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.Option.some(1),
        ffi.Option.some(2),
        ffi.Option.some(3),
        ffi.Option.some(4),
        ffi.Option.some(5),
        ffi.Option.none(),
     ]

    vec = ffi.Vec(c_double, source)
    result = [vec.value(i) for i in range(0, 6)]

    assert result == expect


def test_1():
    assert ffi.mydll.test_fn() == 1234

