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


def test_vec_time():
    offset = ffi.Time(0, 5)
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.value(c_double, 1),
        ffi.value(c_double, 2),
        ffi.value(c_double, 3),
        ffi.value(c_double, 4),
        ffi.value(c_double, 5),
        ffi.not_fixed(c_double),
     ]

    vec = ffi.Vec(ffi.Time, c_double, source, offset)
    result = [vec.value(offset + i) for i in range(0, 6)]
    assert result == expect
    cached_vec = ffi.Cached(ffi.Time, c_double, 10, vec)
    result = [cached_vec.value(offset + i) for i in range(0, 6)]
    assert result == expect


def test_vec_tid():
    offset = 10
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.value(c_double, 1),
        ffi.value(c_double, 2),
        ffi.value(c_double, 3),
        ffi.value(c_double, 4),
        ffi.value(c_double, 5),
        ffi.not_fixed(c_double),
     ]

    vec = ffi.Vec(ffi.TransactionId, c_double, source, offset)
    result = [vec.value(offset + i) for i in range(0, 6)]
    assert result == expect
    cached_vec = ffi.Cached(ffi.TransactionId, c_double, 10, vec)
    result = [cached_vec.value(offset + i) for i in range(0, 6)]
    assert result == expect

def test_storage():
    offset = ffi.Time(0, 5)
    source = [1, 2, None, 4, 5]
    ffi.Option(c_double).none()
    expect = [
        ffi.value(ffi.Option(c_double), ffi.Option(c_double).some(1)),
        ffi.value(ffi.Option(c_double), ffi.Option(c_double).some(2)),
        ffi.value(ffi.Option(c_double), ffi.Option(c_double).none()),
        ffi.value(ffi.Option(c_double), ffi.Option(c_double).some(4)),
        ffi.value(ffi.Option(c_double), ffi.Option(c_double).some(5)),
        ffi.not_fixed(ffi.Option(c_double)),
    ]

    h = ffi.Storage(ffi.Time, c_double, offset)
    for i, v in enumerate(source):
        if v is not None:
            h.add(offset + i, v)
    result = [h.value(offset + i) for i in range(0, 6)]

    assert result == expect

def test_sma():
    offset = ffi.Time(0, 10)
    source = [1, 2, 3, 4, 5]
    expect = [
        ffi.out_of_range(c_double),
        ffi.out_of_range(c_double),
        ffi.value(c_double, 2),
        ffi.value(c_double, 3),
        ffi.value(c_double, 4),
        ffi.not_fixed(c_double),
    ]

    vec = ffi.Vec(ffi.Time, c_double, source, offset)
    sma = ffi.Sma(ffi.Time, c_double, vec, 3)
    result = [sma.value(offset + i) for i in range(0, 6)]

    assert result == expect

def test_cmpl():
    offset = ffi.Time(0, 5)
    source = [1, 2, None, 4, 5]
    expect = [
        ffi.value(c_double, 1),
        ffi.value(c_double, 2),
        ffi.value(c_double, 2),
        ffi.value(c_double, 4),
        ffi.value(c_double, 5),
        ffi.value(c_double, 5),
     ]

    h = ffi.Storage(ffi.Time, c_double, offset)
    for i, v in enumerate(source):
        if v is not None:
            h.add(offset + i, v)
    cmpl = ffi.Cmpl(ffi.Time, c_double, h, 10)
    result = [cmpl.value(offset + i) for i in range(0, 6)]

    assert result == expect

def test_cross():
    offset = ffi.Time("2019-01-01 00:00:00", 60)
    source_1 = [0, 0, 2, 2, 0, 1, 1, 2, 1, 0]
    source_2 = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1]
    expect = [
        ffi.value(c_int, 0),
        ffi.value(c_int, 0),
        ffi.value(c_int, 1),
        ffi.value(c_int, 0),
        ffi.value(c_int, -1),
        ffi.value(c_int, 0),
        ffi.value(c_int, 0),
        ffi.value(c_int, 1),
        ffi.value(c_int, 0),
        ffi.value(c_int, -1),
        ffi.not_fixed(c_int),
    ]

    vec_1 = ffi.Vec(ffi.Time, c_double, source_1, offset)
    vec_2 = ffi.Vec(ffi.Time, c_double, source_2, offset)
    cross = ffi.Cross(ffi.Time, c_double, vec_1, vec_2)
    result = [cross.value(offset + i) for i in range(0, 11)]

    assert result == expect


def test_func():
    offset = ffi.Time("2019-01-01 00:00:00", 60)
    source_1 = [1, 2, 3, 4, 5]
    source_2 = [0, -1, 0, 1, 0]
    expect = [
        ffi.value(c_double, 0),
        ffi.value(c_double, 2),
        ffi.value(c_double, 0),
        ffi.value(c_double, 4),
        ffi.value(c_double, 0),
        ffi.not_fixed(c_double),
     ]
    vec_1 = ffi.Vec(ffi.Time, c_double, source_1, offset)
    vec_2 = ffi.Vec(ffi.Time, c_double, source_2, offset)

    def f(v1, v2):
        return v1 * abs(v2)

    func = ffi.Func(c_double, f, vec_1, vec_2)
    result = [func.value(offset + i) for i in range(0, 6)]

    assert result == expect

def test_slope():
    offset = ffi.Time(0, 10)
    source = [1, 2, 4, 8, 6]
    expect = [
        ffi.out_of_range(c_double),
        ffi.value(c_double, 1),
        ffi.value(c_double, 2),
        ffi.value(c_double, 4),
        ffi.value(c_double, -2),
        ffi.not_fixed(c_double),
    ]

    vec = ffi.Vec(ffi.Time, c_double, source, offset)
    slope = ffi.Slope(ffi.Time, c_double, vec)
    result = [slope.value(offset + i) for i in range(0, 6)]

    assert result == expect

def test_iter_func():
    offset = ffi.Time(0, 10)
    source = [1, 2, 4, 8, 6]
    expect = [
        ffi.value(c_double, 1),
        ffi.value(c_double, 3),
        ffi.value(c_double, 7),
        ffi.value(c_double, 15),
        ffi.value(c_double, 21),
        ffi.not_fixed(c_double),
    ]

    s = 0
    def func(v):
        nonlocal s
        s += v
        return s

    vec = ffi.Vec(ffi.Time, c_double, source, offset)
    f = ffi.IterFunc(ffi.Time, c_double, c_double, vec, offset, func)

    assert f.value(offset + 4) == ffi.value(c_double, 21)

    result = [f.value(offset + i) for i in range(0, 6)]
    assert result == expect


# # # def test_trailing_stop():
# # #     offset = ffi.Time("2019-01-01 00:00:00", 60)
# # #     source_price = [1, 2, -3, 8, 3]
# # #     source_position = [1, 1, 1, 1, 1]
# # #     expect = [
# # #         ffi.Option(c_int).some(0),
# # #         ffi.Option(c_int).some(0),
# # #         ffi.Option(c_int).some(1),
# # #         ffi.Option(c_int).some(0),
# # #         ffi.Option(c_int).some(1),
# # #         ffi.Option(c_int).none(),
# # #     ]

# # #     vec_price = ffi.Vec(offset, c_double, source_price)
# # #     hash_position = ffi.Hash(ffi.SimplePosition, 5)
# # #     for i, v in enumerate(source_position):
# # #         if v is not None:
# # #             hash_position.set(offset + i, v)
# # #     trailing_stop = ffi.TrailingStop(c_double, vec_price, hash_position, 4.0)
# # #     result = [trailing_stop.value(offset + i) for i in range(0, 6)]

# # #     assert result == expect
