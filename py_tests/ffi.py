import os
from ctypes import *

dirname = os.path.dirname(os.path.abspath(__file__))
mydll = cdll.LoadLibrary("{}/../target/debug/libstrategy.dylib".format(dirname))

class Option(Structure):
    _fields_ = [
        ("is_some", c_byte),
        ("value", c_double),
    ]

    @staticmethod
    def some(value):
        return Option(1, value)

    @staticmethod
    def none():
        return Option(0, 0)

    def __eq__(self, other):
        if other is None or not isinstance(other, Option):
            return False
        return self.is_some == other.is_some and \
            self.value == other.value

    def __repr__(self):
        if self.is_some:
            return "Some({})".format(str(self.value))
        else:
            return "None"


mydll.vec_new_f64.argtypes = [POINTER(c_double), c_int]
mydll.vec_new_f64.restype = c_void_p

mydll.vec_value_f64.argtypes = [c_void_p, c_int]
# mydll.vec_value_f64.restype = c_double
mydll.vec_value_f64.restype = Option

class Vec_f64:
    def __init__(self, vec):
        length = len(vec)
        arr = (c_double * length)(*vec)
        ptr = POINTER(c_double)(arr)
        self.ptr = getattr(mydll, "vec_new_f64")(ptr, length)

    def value(self, i):
        return mydll.vec_value_f64(self.ptr, i)

def get_rust_type(T):
    if T == c_double:
        return "f64"

class Vec:
    def __init__(self, T, vec):
        self.T = T
        length = len(vec)
        arr = (T * length)(*vec)
        ptr = POINTER(T)(arr)
        self.ptr = getattr(mydll, "vec_new_{}".format(get_rust_type(self.T)))(ptr, length)

    def value(self, i):
        return getattr(mydll, "vec_value_{}".format(get_rust_type(self.T)))(self.ptr, i)


# def Vec(T):
#     if T == c_double:
#         return Vec_f64


# mydll.vec_new_f64.restype = POINTER(c_int)

