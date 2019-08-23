import os
from ctypes import *

type_map = {
    c_double: "f64",
}

def default(T):
    return 0

def get_rust_type(T):
    if T in type_map:
        return type_map[T]
    else:
        # TODO: raise Error
        pass

dirname = os.path.dirname(os.path.abspath(__file__))
mydll = cdll.LoadLibrary("{}/../target/debug/libstrategy.dylib".format(dirname))

def Option_eq(self, other):
    if other is None or not isinstance(other, self.__class__):
        return False
    if self.is_some == other.is_some:
        if self.is_some:
            return self.value == other.value
        else:
            return True
    else:
        return False

def Option_repr(self):
    if self.is_some:
        return "Some({})".format(str(self.value))
    else:
        return "None"

option_types = {}
for T, t_str in type_map.items():
    option_t_str = "Option_{}".format(t_str)
    option_types[T] = type(option_t_str, (Structure,), {
        "__eq__": Option_eq,
        "__repr__": Option_repr,
    })
    option_types[T]._fields_ = [
        ("is_some", c_byte),
        ("value", T),
    ]
    option_types[T].some = lambda v: option_types[T](1, v)
    option_types[T].none = lambda : option_types[T](0, default(T))
    option_types[T].T = T

def Option(T):
    if T in option_types:
        return option_types[T]
    else:
        pass

class Vec:
    for T, t_str in type_map.items():
        getattr(mydll, "vec_new_{}".format(t_str)).argtypes = [POINTER(T), c_int]
        getattr(mydll, "vec_new_{}".format(t_str)).restype = c_void_p
        getattr(mydll, "vec_destroy_{}".format(t_str)).argtypes = [c_void_p]
        getattr(mydll, "vec_destroy_{}".format(t_str)).restype = None
        getattr(mydll, "vec_value_{}".format(t_str)).argtypes = [c_void_p, c_int]
        getattr(mydll, "vec_value_{}".format(t_str)).restype = Option(T)

    def __init__(self, T, vec):
        self.T = T
        length = len(vec)
        arr = (T * length)(*vec)
        ptr = POINTER(T)(arr)
        self.ptr = getattr(mydll, "vec_new_{}".format(get_rust_type(self.T)))(ptr, length)

    def value(self, i):
        return getattr(mydll, "vec_value_{}".format(get_rust_type(self.T)))(self.ptr, i)

    def __del__(self):
        getattr(mydll, "vec_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None


