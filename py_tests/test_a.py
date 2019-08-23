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


def test_vec():
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.Option(c_double).some(1),
        ffi.Option(c_double).some(2),
        ffi.Option(c_double).some(3),
        ffi.Option(c_double).some(4),
        ffi.Option(c_double).some(5),
        ffi.Option(c_double).none(),
     ]

    vec = ffi.Vec(c_double, source)
    result = [vec.value(i) for i in range(0, 6)]

    assert result == expect

def test_sma():
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.Option(c_double).none(),
        ffi.Option(c_double).none(),
        ffi.Option(c_double).some(2),
        ffi.Option(c_double).some(3),
        ffi.Option(c_double).some(4),
        ffi.Option(c_double).none(),
    ]

    vec = ffi.Vec(c_double, source)
    sma = ffi.Sma(c_double, vec, 3)
    result = [sma.value(i) for i in range(0, 6)]

    assert result == expect
