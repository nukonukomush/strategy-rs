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
    offset = ffi.Time(0, 5)
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.Option(c_double).some(1),
        ffi.Option(c_double).some(2),
        ffi.Option(c_double).some(3),
        ffi.Option(c_double).some(4),
        ffi.Option(c_double).some(5),
        ffi.Option(c_double).none(),
     ]

    vec = ffi.Vec(offset, c_double, source)
    cached_vec = ffi.Cached(c_double, 10, vec)
    result = [cached_vec.value(offset + i) for i in range(0, 6)]

    assert result == expect

def test_sma():
    offset = ffi.Time(0, 10)
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.Option(c_double).none(),
        ffi.Option(c_double).none(),
        ffi.Option(c_double).some(2),
        ffi.Option(c_double).some(3),
        ffi.Option(c_double).some(4),
        ffi.Option(c_double).none(),
    ]

    vec = ffi.Vec(offset, c_double, source)
    sma = ffi.Sma(c_double, vec, 3)
    cached_sma = ffi.Cached(c_double, 10, sma)
    result = [cached_sma.value(offset + i) for i in range(0, 6)]

    assert result == expect

def test_cross():
    offset = ffi.Time("2019-01-01 00:00:00", 60)
    source_1 = [0, 0, 2, 2, 0, 1, 1, 2, 1, 0]
    source_2 = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
    expect = [
        ffi.Option(c_int).some(0),
        ffi.Option(c_int).some(0),
        ffi.Option(c_int).some(1),
        ffi.Option(c_int).some(0),
        ffi.Option(c_int).some(-1),
        ffi.Option(c_int).some(0),
        ffi.Option(c_int).some(0),
        ffi.Option(c_int).some(1),
        ffi.Option(c_int).some(0),
        ffi.Option(c_int).some(-1),
        ffi.Option(c_int).none(),
    ]

    vec_1 = ffi.Vec(offset, c_double, source_1)
    vec_2 = ffi.Vec(offset, c_double, source_2)
    cross = ffi.Cross(c_double, vec_1, vec_2)
    result = [cross.value(offset + i) for i in range(0, 11)]

    assert result == expect
