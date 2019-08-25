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
# mydll = cdll.LoadLibrary("{}/../target/debug/libstrategy.dylib".format(dirname))
mydll = cdll.LoadLibrary("{}/libstrategy.dylib".format(dirname))

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

class Time(Structure):
    _fields_ = [
        ("time", c_longlong),
        ("granularity", c_longlong),
    ]

    def __add__(self, other):
        if type(other) == int:
            return Time(self.time + self.granularity * other, self.granularity)
        else:
            return None

class Ptr(Structure):
    _fields_ = [
        ("b_ptr", c_void_p),
        ("t_ptr", c_void_p),
    ]

getattr(mydll, "indicator_value_{}".format("f64")).argtypes = [c_void_p, Time]
getattr(mydll, "indicator_value_{}".format("f64")).restype = Option(c_double)

class Vec:
    # for T, t_str in type_map.items():
    #     getattr(mydll, "vec_new_{}".format(t_str)).argtypes = [Time, POINTER(T), c_int]
    #     getattr(mydll, "vec_new_{}".format(t_str)).restype = Ptr
    #     getattr(mydll, "vec_destroy_{}".format(t_str)).argtypes = [Ptr]
    #     getattr(mydll, "vec_destroy_{}".format(t_str)).restype = None

    for T, t_str in {
        c_double: "f64",
    }.items():
        getattr(mydll, "vec_new_{}".format(t_str)).argtypes = [Time, POINTER(T), c_int]
        getattr(mydll, "vec_new_{}".format(t_str)).restype = Ptr
        getattr(mydll, "vec_destroy_{}".format(t_str)).argtypes = [Ptr]
        getattr(mydll, "vec_destroy_{}".format(t_str)).restype = None

    def __init__(self, offset, T, vec):
        self.T = T
        length = len(vec)
        arr = (T * length)(*vec)
        ptr = POINTER(T)(arr)
        self.ptr = getattr(mydll, "vec_new_{}".format(get_rust_type(self.T)))(offset, ptr, length)

    def value(self, i):
        return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.ptr.t_ptr, i)

    def __del__(self):
        getattr(mydll, "vec_destroy_{}".format(get_rust_type(self.T)))(self.ptr)
        self.ptr = None

# class Sma:
#     for T, t_str in type_map.items():
#         getattr(mydll, "sma_new_{}".format(t_str)).argtypes = [c_void_p, c_int]
#         getattr(mydll, "sma_new_{}".format(t_str)).restype = c_void_p
#         getattr(mydll, "sma_trait_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "sma_trait_{}".format(t_str)).restype = c_void_p
#         getattr(mydll, "sma_destroy_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "sma_destroy_{}".format(t_str)).restype = None

#     def __init__(self, T, source, period):
#         self.T = T
#         self.b_ptr = getattr(mydll, "sma_new_{}".format(get_rust_type(self.T)))(source.t_ptr, period)
#         self.t_ptr = getattr(mydll, "sma_trait_{}".format(get_rust_type(self.T)))(self.b_ptr)

#     def value(self, i):
#         return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.t_ptr, i)

#     def __del__(self):
#         getattr(mydll, "sma_destroy_{}".format(get_rust_type(self.T)))(self.b_ptr)
#         self.b_ptr = None
#         getattr(mydll, "indicator_destroy_{}".format(get_rust_type(self.T)))(self.t_ptr)
#         self.t_ptr = None

# class Cached:
#     for T, t_str in type_map.items():
#         getattr(mydll, "cached_new_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "cached_new_{}".format(t_str)).restype = c_void_p
#         getattr(mydll, "cached_trait_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "cached_trait_{}".format(t_str)).restype = c_void_p
#         getattr(mydll, "cached_destroy_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "cached_destroy_{}".format(t_str)).restype = None

#     def __init__(self, T, source):
#         self.T = T
#         self.b_ptr = getattr(mydll, "cached_new_{}".format(get_rust_type(self.T)))(source.t_ptr)
#         self.t_ptr = getattr(mydll, "cached_trait_{}".format(get_rust_type(self.T)))(self.b_ptr)

#     def value(self, i):
#         return getattr(mydll, "indicator_value_{}".format(get_rust_type(self.T)))(self.t_ptr, i)

#     def __del__(self):
#         getattr(mydll, "cached_destroy_{}".format(get_rust_type(self.T)))(self.b_ptr)
#         self.b_ptr = None
#         getattr(mydll, "indicator_destroy_{}".format(get_rust_type(self.T)))(self.t_ptr)
#         self.t_ptr = None

# # OrderingValue = c_int
# # getattr(mydll, "indicator_value_{}".format("ordering")).argtypes = [c_void_p, c_int]
# # getattr(mydll, "indicator_value_{}".format("ordering")).restype = Option(OrderingValue)
# # getattr(mydll, "indicator_destroy_{}".format("ordering")).argtypes = [c_void_p]
# # getattr(mydll, "indicator_destroy_{}".format("ordering")).restype = None
# # class Ordering:
# #     for T, t_str in type_map.items():
# #         getattr(mydll, "ordering_new_{}".format(t_str)).argtypes = [c_void_p, c_void_p]
# #         getattr(mydll, "ordering_new_{}".format(t_str)).restype = c_void_p
# #         getattr(mydll, "ordering_trait_{}".format(t_str)).argtypes = [c_void_p]
# #         getattr(mydll, "ordering_trait_{}".format(t_str)).restype = c_void_p
# #         getattr(mydll, "ordering_destroy_{}".format(t_str)).argtypes = [c_void_p]
# #         getattr(mydll, "ordering_destroy_{}".format(t_str)).restype = None

# #     def __init__(self, T, source):
# #         self.T = T
# #         self.b_ptr = getattr(mydll, "ordering_new_{}".format(get_rust_type(self.T)))(source.t_ptr)
# #         self.t_ptr = getattr(mydll, "ordering_trait_{}".format(get_rust_type(self.T)))(self.b_ptr)

# #     def value(self, i):
# #         return getattr(mydll, "indicator_value_ordering")(self.t_ptr, i)

# #     def __del__(self):
# #         getattr(mydll, "ordering_destroy_{}".format(get_rust_type(self.T)))(self.b_ptr)
# #         self.b_ptr = None
# #         getattr(mydll, "indicator_destroy_{}".format(get_rust_type(self.T)))(self.t_ptr)
# #         self.t_ptr = None

# CrossState = c_int
# # getattr(mydll, "indicator_value_{}".format("cross")).argtypes = [c_void_p, c_int]
# # getattr(mydll, "indicator_value_{}".format("cross")).restype = Option(CrossState)
# # getattr(mydll, "indicator_destroy_{}".format("cross")).argtypes = [c_void_p]
# # getattr(mydll, "indicator_destroy_{}".format("cross")).restype = None
# class Cross:
#     for T, t_str in type_map.items():
#         getattr(mydll, "cross_new_{}".format(t_str)).argtypes = [c_void_p, c_void_p]
#         getattr(mydll, "cross_new_{}".format(t_str)).restype = c_void_p
#         getattr(mydll, "cross_trait_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "cross_trait_{}".format(t_str)).restype = c_void_p
#         getattr(mydll, "cross_destroy_{}".format(t_str)).argtypes = [c_void_p]
#         getattr(mydll, "cross_destroy_{}".format(t_str)).restype = None

#     def __init__(self, T, source_1, source_2):
#         self.T = T
#         self.b_ptr = getattr(mydll, "cross_new_{}".format(get_rust_type(self.T)))(source_1.t_ptr, source_2.t_ptr)
#         self.t_ptr = getattr(mydll, "cross_trait_{}".format(get_rust_type(self.T)))(self.b_ptr)

#     def value(self, i):
#         return getattr(mydll, "indicator_value_cross")(self.t_ptr, i)

#     def __del__(self):
#         getattr(mydll, "cross_destroy_{}".format(get_rust_type(self.T)))(self.b_ptr)
#         self.b_ptr = None
#         getattr(mydll, "indicator_destroy_{}".format("cross"))(self.t_ptr)
#         self.t_ptr = None
