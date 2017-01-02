extern crate libc;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct roaring_bitmap_s {
    pub high_low_container: roaring_array_s,
    pub copy_on_write: bool
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct roaring_array_s {
    pub size: ::libc::int32_t,
    pub allocation_size: ::libc::int32_t,
    pub containers: *mut *mut ::std::os::raw::c_void,
    pub keys: *mut ::libc::uint16_t,
    pub typecodes: *mut ::libc::uint8_t
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct roaring_uint32_iterator_s {
    pub parent: *const roaring_bitmap_s,
    pub container_index: ::libc::int32_t,
    pub in_container_index: ::libc::int32_t,
    pub run_index: ::libc::int32_t,
    pub in_run_index: ::libc::uint32_t,
    pub current_value: ::libc::uint32_t,
    pub has_value: bool
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct roaring_statistics_s {
    pub n_containers: ::libc::uint32_t,
    pub n_array_containers: ::libc::uint32_t,
    pub n_run_containers: ::libc::uint32_t,
    pub n_bitset_containers: ::libc::uint32_t,
    pub n_values_array_containers: ::libc::uint32_t,
    pub n_values_run_containers: ::libc::uint32_t,
    pub n_values_bitset_containers: ::libc::uint32_t,
    pub n_bytes_array_containers: ::libc::uint32_t,
    pub n_bytes_run_containers: ::libc::uint32_t,
    pub n_bytes_bitset_containers: ::libc::uint32_t,
    pub max_value: ::libc::uint32_t,
    pub min_value: ::libc::uint32_t,
    pub sum_value: ::libc::uint64_t,
    pub cardinality: ::libc::uint64_t
}

impl Default for roaring_statistics_s {
    fn default() -> roaring_statistics_s { unsafe { std::mem::zeroed() } }
}

#[link(name = "roaring", kind = "static")]
extern "C" {
    pub fn roaring_bitmap_create() -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_create_with_capacity(cap: ::libc::uint32_t) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_of_ptr(n_args: ::libc::size_t, vals: *const ::libc::uint32_t) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_of(n: ::libc::size_t, ...) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_copy(r: *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_printf(ra: *const roaring_bitmap_s);
    pub fn roaring_bitmap_and(x1: *const roaring_bitmap_s, x2: *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_and_inplace(x1: *mut roaring_bitmap_s, x2: *const roaring_bitmap_s);
    pub fn roaring_bitmap_or(x1: *const roaring_bitmap_s, x2: *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_or_inplace(x1: *mut roaring_bitmap_s, x2: *const roaring_bitmap_s);
    pub fn roaring_bitmap_or_many(number: ::libc::size_t, x: *mut *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_or_many_heap(number: ::libc::uint32_t, x: *mut *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_xor(x1: *const roaring_bitmap_s, x2: *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_xor_inplace(x1: *mut roaring_bitmap_s, x2: *const roaring_bitmap_s);
    pub fn roaring_bitmap_flip(x1: *const roaring_bitmap_s,
                               range_start: ::libc::uint64_t, range_end: ::libc::uint64_t)
                               -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_flip_inplace(x1: *mut roaring_bitmap_s,
                                       range_start: ::libc::uint64_t,
                                       range_end: ::libc::uint64_t);
    pub fn roaring_bitmap_xor_many(number: ::libc::size_t, x: *mut *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_andnot(x1: *const roaring_bitmap_s, x2: *const roaring_bitmap_s) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_andnot_inplace(x1: *mut roaring_bitmap_s, x2: *const roaring_bitmap_s);
    pub fn roaring_bitmap_free(r: *mut roaring_bitmap_s);
    pub fn roaring_bitmap_add(r: *mut roaring_bitmap_s, x: ::libc::uint32_t);
    pub fn roaring_bitmap_add_many(r: *mut roaring_bitmap_s, n_args: ::libc::size_t, vals: *const ::libc::uint32_t);
    pub fn roaring_bitmap_remove(r: *mut roaring_bitmap_s, x: ::libc::uint32_t);
    pub fn roaring_bitmap_contains(r: *const roaring_bitmap_s, x: ::libc::uint32_t) -> bool;
    pub fn roaring_bitmap_get_cardinality(ra: *const roaring_bitmap_s) -> ::libc::uint64_t;
    pub fn roaring_bitmap_to_uint32_array(ra: *const roaring_bitmap_s, cardinality: *mut ::libc::uint32_t);
    pub fn roaring_bitmap_remove_run_compression(r: *mut roaring_bitmap_s) -> bool;
    pub fn roaring_bitmap_run_optimize(r: *mut roaring_bitmap_s) -> bool;
    pub fn roaring_bitmap_serialize(ra: *const roaring_bitmap_s, buf: *mut ::libc::c_char) -> ::libc::size_t;
    pub fn roaring_bitmap_deserialize(buf: *const ::libc::c_void) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_portable_deserialize(buf: *const ::libc::c_char) -> *mut roaring_bitmap_s;
    pub fn roaring_bitmap_portable_size_in_bytes(ra: *const roaring_bitmap_s) -> ::libc::size_t;
    pub fn roaring_bitmap_portable_serialize(ra: *const roaring_bitmap_s, buf: *mut ::libc::c_char) -> ::libc::size_t;
    pub fn roaring_bitmap_is_empty(ra: *const roaring_bitmap_s) -> bool;
    pub fn roaring_bitmap_equals(ra1: *const roaring_bitmap_s, ra2: *const roaring_bitmap_s) -> bool;
    pub fn roaring_bitmap_is_subset(ra1: *const roaring_bitmap_s, ra2: *const roaring_bitmap_s) -> bool;
    pub fn roaring_bitmap_is_strict_subset(ra1: *const roaring_bitmap_s, ra2: *const roaring_bitmap_s) -> bool;
    pub fn roaring_bitmap_statistics(ra: *const roaring_bitmap_s, stat: *mut roaring_statistics_s);
    pub fn roaring_create_iterator(ra: *const roaring_bitmap_s) -> *mut roaring_uint32_iterator_s;
    pub fn roaring_advance_uint32_iterator(it: *mut roaring_uint32_iterator_s) -> bool;
    pub fn roaring_free_uint32_iterator(it: *mut roaring_uint32_iterator_s);
}
