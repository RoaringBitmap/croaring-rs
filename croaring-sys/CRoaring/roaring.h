/* auto-generated on Fri Sep  2 21:26:08 EDT 2016. Do not edit! */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/portability.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/portability.h"
/*
 * portability.h
 *
 */

#ifndef INCLUDE_PORTABILITY_H_
#define INCLUDE_PORTABILITY_H_

#include <stdint.h>

#if __SIZEOF_LONG_LONG__ != 8
#error This code assumes  64-bit long longs (by use of the GCC intrinsics). Your system is not currently supported.
#endif

#if defined(_MSC_VER)
#define __restrict__ __restrict
#endif

// unless DISABLEAVX was defined, if we have AVX2 and BMI2, we enable AVX
#if (!defined(USEAVX)) && (!defined(DISABLEAVX)) && (defined(__AVX2__)) && \
    (defined(__BMI2__))
#define USEAVX
#endif

// if USEAVX was somehow defined and we lack either AVX2 or BMI2, we disable it
#if defined(USEAVX) && ((!defined(__AVX2__)) || (!defined(__BMI2__)))
#undef USEAVX
#endif

#if defined(USEAVX) || defined(__x86_64__) || defined(_M_X64)
// we have an x64 processor
#define IS_X64
// we include the intrinsic header
#ifdef _MSC_VER
/* Microsoft C/C++-compatible compiler */
#include <intrin.h>
#else
/* Pretty much anything else. */
#include <x86intrin.h>
#endif
#endif

// if we have AVX, then we use BMI optimizations
#if defined(USEAVX)
#define USE_BMI  // we assume that AVX2 and BMI go hand and hand
#define USEAVX2FORDECODING            // optimization
#define ROARING_VECTOR_UNION_ENABLED  // vector unions (optimization)
#endif

#if defined(_MSC_VER)
#define ALIGNED(x) __declspec(align(x))
#else
#if defined(__GNUC__)
#define ALIGNED(x) __attribute__((aligned(x)))
#endif
#endif

#ifdef __GNUC__
#define WARN_UNUSED __attribute__((warn_unused_result))
#else
#define WARN_UNUSED
#endif

#define IS_BIG_ENDIAN (*(uint16_t *)"\0\xff" < 0x100)

static inline int hamming(uint64_t x) {
#if defined(IS_X64) && defined(__POPCNT__)
    return _mm_popcnt_u64(x);
#else
    // won't work under visual studio, but hopeful we have _mm_popcnt_u64 in
    // many cases
    return __builtin_popcountll(x);
#endif
}

#ifndef UINT64_C
#define UINT64_C(c) (c##ULL)
#endif

#ifndef UINT32_C
#define UINT32_C(c) (c##UL)
#endif

#endif /* INCLUDE_PORTABILITY_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/portability.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/perfparameters.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/perfparameters.h"
#ifndef PERFPARAMETERS_H_
#define PERFPARAMETERS_H_

#include <stdbool.h>

/**
During lazy computations, we can transform array containers into bitset
containers as
long as we can expect them to have  ARRAY_LAZY_LOWERBOUND values.
*/
enum { ARRAY_LAZY_LOWERBOUND = 1024 };

/* default initial size of a run container */
enum { RUN_DEFAULT_INIT_SIZE = 4 };

/* default initial size of an array container */
enum { ARRAY_DEFAULT_INIT_SIZE = 16 };

/* automatic bitset conversion during lazy or */
#ifndef LAZY_OR_BITSET_CONVERSION
#define LAZY_OR_BITSET_CONVERSION true
#endif

#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/perfparameters.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/array_util.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/array_util.h"
#ifndef ARRAY_UTIL_H
#define ARRAY_UTIL_H

#include <stddef.h>  // for size_t
#include <stdint.h>

/*
 *  good old binary search
 */
inline int32_t binarySearch(const uint16_t *array, int32_t lenarray,
                                   uint16_t ikey) {
    int32_t low = 0;
    int32_t high = lenarray - 1;
    while (low <= high) {
        int32_t middleIndex = (low + high) >> 1;
        uint16_t middleValue = array[middleIndex];
        if (middleValue < ikey) {
            low = middleIndex + 1;
        } else if (middleValue > ikey) {
            high = middleIndex - 1;
        } else {
            return middleIndex;
        }
    }
    return -(low + 1);
}

/**
 * Galloping search
 */
static inline int32_t advanceUntil(const uint16_t *array, int32_t pos,
                                   int32_t length, uint16_t min) {
    int32_t lower = pos + 1;

    if ((lower >= length) || (array[lower] >= min)) {
        return lower;
    }

    int32_t spansize = 1;

    while ((lower + spansize < length) && (array[lower + spansize] < min)) {
        spansize <<= 1;
    }
    int32_t upper = (lower + spansize < length) ? lower + spansize : length - 1;

    if (array[upper] == min) {
        return upper;
    }
    if (array[upper] < min) {
        // means
        // array
        // has no
        // item
        // >= min
        // pos = array.length;
        return length;
    }

    // we know that the next-smallest span was too small
    lower += (spansize >> 1);

    int32_t mid = 0;
    while (lower + 1 != upper) {
        mid = (lower + upper) >> 1;
        if (array[mid] == min) {
            return mid;
        } else if (array[mid] < min) {
            lower = mid;
        } else {
            upper = mid;
        }
    }
    return upper;
}

/**
 * From Schlegel et al., Fast Sorted-Set Intersection using SIMD Instructions
 * Optimized by D. Lemire on May 3rd 2013
 *
 * C should have capacity greater than the minimum of s_1 and s_b + 8
 * where 8 is sizeof(__m128i)/sizeof(uint16_t).
 */
int32_t intersect_vector16(const uint16_t *A, size_t s_a, const uint16_t *B,
                           size_t s_b, uint16_t *C);

/* Computes the intersection between one small and one large set of uint16_t.
 * Stores the result into buffer and return the number of elements. */
int32_t intersect_skewed_uint16(const uint16_t *small, size_t size_s,
                                const uint16_t *large, size_t size_l,
                                uint16_t *buffer);

/**
 * Generic intersection function. Passes unit tests.
 */
int32_t intersect_uint16(const uint16_t *A, const size_t lenA,
                         const uint16_t *B, const size_t lenB, uint16_t *out);

/**
 * Generic union function.
 */
size_t union_uint16(const uint16_t *set_1, size_t size_1, const uint16_t *set_2,
                    size_t size_2, uint16_t *buffer);

/**
 * Generic intersection function.
 */
size_t intersection_uint32(const uint32_t *A, const size_t lenA,
                           const uint32_t *B, const size_t lenB, uint32_t *out);

/**
 * Generic intersection function, returns just the cardinality.
 */
size_t intersection_uint32_card(const uint32_t *A, const size_t lenA,
                                const uint32_t *B, const size_t lenB);

/**
 * Generic union function.
 */
size_t union_uint32(const uint32_t *set_1, size_t size_1, const uint32_t *set_2,
                    size_t size_2, uint32_t *buffer);

/**
 * A fast SSE-based union function.
 */
uint32_t union_vector16(const uint16_t *set_1, uint32_t size_1,
                        const uint16_t *set_2, uint32_t size_2,
                        uint16_t *buffer);
/**
 * Generic union function, returns just the cardinality.
 */
size_t union_uint32_card(const uint32_t *set_1, size_t size_1,
                         const uint32_t *set_2, size_t size_2);

#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/array_util.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/roaring_types.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/roaring_types.h"
/*
  Typedefs used by various components
*/

#ifndef ROARING_TYPES_H
#define ROARING_TYPES_H

typedef bool (*roaring_iterator)(uint32_t value, void *param);

/**
*  (For advanced users.)
* The roaring_statistics_t can be used to collect detailed statistics about
* the composition of a roaring bitmap.
*/
typedef struct roaring_statistics_s {
    uint32_t n_containers; /* number of containers */

    uint32_t n_array_containers;  /* number of array containers */
    uint32_t n_run_containers;    /* number of run containers */
    uint32_t n_bitset_containers; /* number of bitmap containers */

    uint32_t
        n_values_array_containers;    /* number of values in array containers */
    uint32_t n_values_run_containers; /* number of values in run containers */
    uint32_t
        n_values_bitset_containers; /* number of values in  bitmap containers */

    uint32_t n_bytes_array_containers; /* number of allocated bytes in array
                                          containers */
    uint32_t n_bytes_run_containers;   /* number of allocated bytes in run
                                          containers */
    uint32_t
        n_bytes_bitset_containers; /* number of allocated bytes in  bitmap
                                      containers */

    uint32_t
        max_value; /* the maximal value, undefined if cardinality is zero */
    uint32_t
        min_value; /* the minimal value, undefined if cardinality is zero */
    uint64_t sum_value; /* the sum of all values (could be used to compute
                           average) */

    uint64_t cardinality; /* total number of values stored in the bitmap */

    // and n_values_arrays, n_values_rle, n_values_bitmap
} roaring_statistics_t;

#endif /* ROARING_TYPES_H */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/roaring_types.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/utilasm.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/utilasm.h"
/*
 * utilasm.h
 *
 */

#ifndef INCLUDE_UTILASM_H_
#define INCLUDE_UTILASM_H_


#if defined(USE_BMI)
#define ASMBITMANIPOPTIMIZATION  // optimization flag

#define ASM_SHIFT_RIGHT(srcReg, bitsReg, destReg) \
    __asm volatile("shrx %1, %2, %0"              \
                   : "=r"(destReg)                \
                   :             /* write */      \
                   "r"(bitsReg), /* read only */  \
                   "r"(srcReg)   /* read only */  \
                   )

#define ASM_INPLACESHIFT_RIGHT(srcReg, bitsReg)  \
    __asm volatile("shrx %1, %0, %0"             \
                   : "+r"(srcReg)                \
                   :            /* read/write */ \
                   "r"(bitsReg) /* read only */  \
                   )

#define ASM_SHIFT_LEFT(srcReg, bitsReg, destReg) \
    __asm volatile("shlx %1, %2, %0"             \
                   : "=r"(destReg)               \
                   :             /* write */     \
                   "r"(bitsReg), /* read only */ \
                   "r"(srcReg)   /* read only */ \
                   )
// set bit at position testBit within testByte to 1 and
// copy cmovDst to cmovSrc if that bit was previously clear
#define ASM_SET_BIT_INC_WAS_CLEAR(testByte, testBit, count) \
    __asm volatile(                                         \
        "bts %2, %0\n"                                      \
        "sbb $-1, %1\n"                                     \
        : "+r"(testByte), /* read/write */                  \
          "+r"(count)                                       \
        :            /* read/write */                       \
        "r"(testBit) /* read only */                        \
        )

#define ASM_CLEAR_BIT_DEC_WAS_SET(testByte, testBit, count) \
    __asm volatile(                                         \
        "btr %2, %0\n"                                      \
        "sbb $0, %1\n"                                      \
        : "+r"(testByte), /* read/write */                  \
          "+r"(count)                                       \
        :            /* read/write */                       \
        "r"(testBit) /* read only */                        \
        )

#define ASM_BT64(testByte, testBit, count) \
    __asm volatile(                        \
        "bt %2,%1\n"                       \
        "sbb %0,%0"                        \
        : "=r"(count)                      \
        :              /* write */         \
        "r"(testByte), /* read only */     \
        "r"(testBit)   /* read only */     \
        )

#endif  // USE_BMI
#endif  /* INCLUDE_UTILASM_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/utilasm.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/bitset_util.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/bitset_util.h"
#ifndef BITSET_UTIL_H
#define BITSET_UTIL_H

#include <stdint.h>


/*
 * Set all bits in indexes [begin,end) to true.
 */
static inline void bitset_set_range(uint64_t *bitmap, uint32_t start,
                                    uint32_t end) {
    if (start == end) return;
    uint32_t firstword = start / 64;
    uint32_t endword = (end - 1) / 64;
    if (firstword == endword) {
        bitmap[firstword] |= ((~UINT64_C(0)) << (start % 64)) &
                             ((~UINT64_C(0)) >> ((-end) % 64));
        return;
    }
    bitmap[firstword] |= (~UINT64_C(0)) << (start % 64);
    for (uint32_t i = firstword + 1; i < endword; i++) bitmap[i] = ~UINT64_C(0);
    bitmap[endword] |= (~UINT64_C(0)) >> ((-end) % 64);
}

/*
 * Set all bits in indexes [begin,begin+lenminusone] to true.
 */
static inline void bitset_set_lenrange(uint64_t *bitmap, uint32_t start,
                                       uint32_t lenminusone) {
    uint32_t firstword = start / 64;
    uint32_t endword = (start + lenminusone) / 64;
    if (firstword == endword) {
        bitmap[firstword] |= ((~UINT64_C(0)) >> ((63 - lenminusone)))
                             << (start % 64);
        return;
    }
    uint64_t temp = bitmap[endword];
    bitmap[firstword] |= (~UINT64_C(0)) << (start % 64);
    for (uint32_t i = firstword + 1; i < endword; i += 2)
        bitmap[i] = bitmap[i + 1] = ~UINT64_C(0);
    bitmap[endword] =
        temp | (~UINT64_C(0)) >> ((-start - lenminusone - 1) % 64);
}

/*
 * Flip all the bits in indexes [begin,end).
 */
static inline void bitset_flip_range(uint64_t *bitmap, uint32_t start,
                                     uint32_t end) {
    if (start == end) return;
    uint32_t firstword = start / 64;
    uint32_t endword = (end - 1) / 64;
    bitmap[firstword] ^= ~((~UINT64_C(0)) << (start % 64));
    for (uint32_t i = firstword; i < endword; i++) bitmap[i] = ~bitmap[i];
    bitmap[endword] ^= ((~UINT64_C(0)) >> ((-end) % 64));
}

/*
 * Set all bits in indexes [begin,end) to false.
 */
static inline void bitset_reset_range(uint64_t *bitmap, uint32_t start,
                                      uint32_t end) {
    if (start == end) return;
    uint32_t firstword = start / 64;
    uint32_t endword = (end - 1) / 64;
    if (firstword == endword) {
        bitmap[firstword] &= ~(((~UINT64_C(0)) << (start % 64)) &
                               ((~UINT64_C(0)) >> ((-end) % 64)));
        return;
    }
    bitmap[firstword] &= ~((~UINT64_C(0)) << (start % 64));
    for (uint32_t i = firstword + 1; i < endword; i++) bitmap[i] = UINT64_C(0);
    bitmap[endword] &= ~((~UINT64_C(0)) >> ((-end) % 64));
}

/*
 * Given a bitset containing "length" 64-bit words, write out the position
 * of all the set bits to "out", values start at "base".
 *
 * The "out" pointer should be sufficient to store the actual number of bits
 * set.
 *
 * Returns how many values were actually decoded.
 *
 * This function should only be expected to be faster than
 * bitset_extract_setbits
 * when the density of the bitset is high.
 *
 * This function uses AVX2 decoding.
 */
size_t bitset_extract_setbits_avx2(uint64_t *bitset, size_t length,
                                   void *vout, size_t outcapacity,
                                   uint32_t base);

/*
 * Given a bitset containing "length" 64-bit words, write out the position
 * of all the set bits to "out", values start at "base".
 *
 * The "out" pointer should be sufficient to store the actual number of bits
 *set.
 *
 * Returns how many values were actually decoded.
 */
size_t bitset_extract_setbits(uint64_t *bitset, size_t length, void *vout,
                              uint32_t base);

/*
 * Given a bitset containing "length" 64-bit words, write out the position
 * of all the set bits to "out" as 16-bit integers, values start at "base" (can
 *be set to zero)
 *
 * The "out" pointer should be sufficient to store the actual number of bits
 *set.
 *
 * Returns how many values were actually decoded.
 *
 * This function should only be expected to be faster than
 *bitset_extract_setbits_uint16
 * when the density of the bitset is high.
 *
 * This function uses SSE decoding.
 */
size_t bitset_extract_setbits_sse_uint16(const uint64_t *bitset, size_t length,
                                         uint16_t *out, size_t outcapacity,
                                         uint16_t base);

/*
 * Given a bitset containing "length" 64-bit words, write out the position
 * of all the set bits to "out",  values start at "base"
 * (can be set to zero)
 *
 * The "out" pointer should be sufficient to store the actual number of bits
 *set.
 *
 * Returns how many values were actually decoded.
 */
size_t bitset_extract_setbits_uint16(const uint64_t *bitset, size_t length,
                                     uint16_t *out, uint16_t base);

/*
 * Given two bitsets containing "length" 64-bit words, write out the position
 * of all the common set bits to "out", values start at "base"
 * (can be set to zero)
 *
 * The "out" pointer should be sufficient to store the actual number of bits
 * set.
 *
 * Returns how many values were actually decoded.
 */
size_t bitset_extract_intersection_setbits_uint16(const uint64_t *bitset1,
                                                  const uint64_t *bitset2,
                                                  size_t length, uint16_t *out,
                                                  uint16_t base);

/*
 * Given a bitset having cardinality card, set all bit values in the list (there
 * are length of them)
 * and return the updated cardinality. This evidently assumes that the bitset
 * already contained data.
 */
uint64_t bitset_set_list_withcard(void *bitset, uint64_t card,
                                  const uint16_t *list, uint64_t length);
/*
 * Given a bitset, set all bit values in the list (there
 * are length of them).
 */
void bitset_set_list(void *bitset, const uint16_t *list, uint64_t length);

/*
 * Given a bitset having cardinality card, unset all bit values in the list
 * (there are length of them)
 * and return the updated cardinality. This evidently assumes that the bitset
 * already contained data.
 */
uint64_t bitset_clear_list(void *bitset, uint64_t card, const uint16_t *list,
                           uint64_t length);

/*
 * Given a bitset having cardinality card, toggle all bit values in the list
 * (there are length of them)
 * and return the updated cardinality. This evidently assumes that the bitset
 * already contained data.
 */

uint64_t bitset_flip_list_withcard(void *bitset, uint64_t card,
                                   const uint16_t *list, uint64_t length);

void bitset_flip_list(void *bitset, const uint16_t *list, uint64_t length);

#ifdef USEAVX
/***
 * BEGIN Harley-Seal popcount functions.
 */

/**
 * Compute the population count of a 256-bit word
 * This is not especially fast, but it is convenient as part of other functions.
 */
static inline __m256i popcount256(__m256i v) {
    const __m256i lookuppos = _mm256_setr_epi8(
        /* 0 */ 4 + 0, /* 1 */ 4 + 1, /* 2 */ 4 + 1, /* 3 */ 4 + 2,
        /* 4 */ 4 + 1, /* 5 */ 4 + 2, /* 6 */ 4 + 2, /* 7 */ 4 + 3,
        /* 8 */ 4 + 1, /* 9 */ 4 + 2, /* a */ 4 + 2, /* b */ 4 + 3,
        /* c */ 4 + 2, /* d */ 4 + 3, /* e */ 4 + 3, /* f */ 4 + 4,

        /* 0 */ 4 + 0, /* 1 */ 4 + 1, /* 2 */ 4 + 1, /* 3 */ 4 + 2,
        /* 4 */ 4 + 1, /* 5 */ 4 + 2, /* 6 */ 4 + 2, /* 7 */ 4 + 3,
        /* 8 */ 4 + 1, /* 9 */ 4 + 2, /* a */ 4 + 2, /* b */ 4 + 3,
        /* c */ 4 + 2, /* d */ 4 + 3, /* e */ 4 + 3, /* f */ 4 + 4);
    const __m256i lookupneg = _mm256_setr_epi8(
        /* 0 */ 4 - 0, /* 1 */ 4 - 1, /* 2 */ 4 - 1, /* 3 */ 4 - 2,
        /* 4 */ 4 - 1, /* 5 */ 4 - 2, /* 6 */ 4 - 2, /* 7 */ 4 - 3,
        /* 8 */ 4 - 1, /* 9 */ 4 - 2, /* a */ 4 - 2, /* b */ 4 - 3,
        /* c */ 4 - 2, /* d */ 4 - 3, /* e */ 4 - 3, /* f */ 4 - 4,

        /* 0 */ 4 - 0, /* 1 */ 4 - 1, /* 2 */ 4 - 1, /* 3 */ 4 - 2,
        /* 4 */ 4 - 1, /* 5 */ 4 - 2, /* 6 */ 4 - 2, /* 7 */ 4 - 3,
        /* 8 */ 4 - 1, /* 9 */ 4 - 2, /* a */ 4 - 2, /* b */ 4 - 3,
        /* c */ 4 - 2, /* d */ 4 - 3, /* e */ 4 - 3, /* f */ 4 - 4);
    const __m256i low_mask = _mm256_set1_epi8(0x0f);

    const __m256i lo = _mm256_and_si256(v, low_mask);
    const __m256i hi = _mm256_and_si256(_mm256_srli_epi16(v, 4), low_mask);
    const __m256i popcnt1 = _mm256_shuffle_epi8(lookuppos, lo);
    const __m256i popcnt2 = _mm256_shuffle_epi8(lookupneg, hi);
    return _mm256_sad_epu8(popcnt1, popcnt2);
}

/**
 * Simple CSA over 256 bits
 */
static inline void CSA(__m256i *h, __m256i *l, __m256i a, __m256i b,
                       __m256i c) {
    const __m256i u = _mm256_xor_si256(a, b);
    *h = _mm256_or_si256(_mm256_and_si256(a, b), _mm256_and_si256(u, c));
    *l = _mm256_xor_si256(u, c);
}

/**
 * Fast Harley-Seal AVX population count function
 */
inline static uint64_t avx2_harley_seal_popcount256(const __m256i *data,
                                                    const uint64_t size) {
    __m256i total = _mm256_setzero_si256();
    __m256i ones = _mm256_setzero_si256();
    __m256i twos = _mm256_setzero_si256();
    __m256i fours = _mm256_setzero_si256();
    __m256i eights = _mm256_setzero_si256();
    __m256i sixteens = _mm256_setzero_si256();
    __m256i twosA, twosB, foursA, foursB, eightsA, eightsB;

    const uint64_t limit = size - size % 16;
    uint64_t i = 0;

    for (; i < limit; i += 16) {
        CSA(&twosA, &ones, ones, _mm256_lddqu_si256(data + i),
            _mm256_lddqu_si256(data + i + 1));
        CSA(&twosB, &ones, ones, _mm256_lddqu_si256(data + i + 2),
            _mm256_lddqu_si256(data + i + 3));
        CSA(&foursA, &twos, twos, twosA, twosB);
        CSA(&twosA, &ones, ones, _mm256_lddqu_si256(data + i + 4),
            _mm256_lddqu_si256(data + i + 5));
        CSA(&twosB, &ones, ones, _mm256_lddqu_si256(data + i + 6),
            _mm256_lddqu_si256(data + i + 7));
        CSA(&foursB, &twos, twos, twosA, twosB);
        CSA(&eightsA, &fours, fours, foursA, foursB);
        CSA(&twosA, &ones, ones, _mm256_lddqu_si256(data + i + 8),
            _mm256_lddqu_si256(data + i + 9));
        CSA(&twosB, &ones, ones, _mm256_lddqu_si256(data + i + 10),
            _mm256_lddqu_si256(data + i + 11));
        CSA(&foursA, &twos, twos, twosA, twosB);
        CSA(&twosA, &ones, ones, _mm256_lddqu_si256(data + i + 12),
            _mm256_lddqu_si256(data + i + 13));
        CSA(&twosB, &ones, ones, _mm256_lddqu_si256(data + i + 14),
            _mm256_lddqu_si256(data + i + 15));
        CSA(&foursB, &twos, twos, twosA, twosB);
        CSA(&eightsB, &fours, fours, foursA, foursB);
        CSA(&sixteens, &eights, eights, eightsA, eightsB);

        total = _mm256_add_epi64(total, popcount256(sixteens));
    }

    total = _mm256_slli_epi64(total, 4);  // * 16
    total = _mm256_add_epi64(
        total, _mm256_slli_epi64(popcount256(eights), 3));  // += 8 * ...
    total = _mm256_add_epi64(
        total, _mm256_slli_epi64(popcount256(fours), 2));  // += 4 * ...
    total = _mm256_add_epi64(
        total, _mm256_slli_epi64(popcount256(twos), 1));  // += 2 * ...
    total = _mm256_add_epi64(total, popcount256(ones));
    for (; i < size; i++)
        total =
            _mm256_add_epi64(total, popcount256(_mm256_lddqu_si256(data + i)));

    return (uint64_t)(_mm256_extract_epi64(total, 0)) +
           (uint64_t)(_mm256_extract_epi64(total, 1)) +
           (uint64_t)(_mm256_extract_epi64(total, 2)) +
           (uint64_t)(_mm256_extract_epi64(total, 3));
}

#define AVXPOPCNTFNC(opname, avx_intrinsic)                                    \
    static inline uint64_t avx2_harley_seal_popcount256_##opname(              \
        const __m256i *data1, const __m256i *data2, const uint64_t size) {     \
        __m256i total = _mm256_setzero_si256();                                \
        __m256i ones = _mm256_setzero_si256();                                 \
        __m256i twos = _mm256_setzero_si256();                                 \
        __m256i fours = _mm256_setzero_si256();                                \
        __m256i eights = _mm256_setzero_si256();                               \
        __m256i sixteens = _mm256_setzero_si256();                             \
        __m256i twosA, twosB, foursA, foursB, eightsA, eightsB;                \
        __m256i A1, A2;                                                        \
        const uint64_t limit = size - size % 16;                               \
        uint64_t i = 0;                                                        \
        for (; i < limit; i += 16) {                                           \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i),                  \
                               _mm256_lddqu_si256(data2 + i));                 \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 1),              \
                               _mm256_lddqu_si256(data2 + i + 1));             \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 2),              \
                               _mm256_lddqu_si256(data2 + i + 2));             \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 3),              \
                               _mm256_lddqu_si256(data2 + i + 3));             \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursA, &twos, twos, twosA, twosB);                           \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 4),              \
                               _mm256_lddqu_si256(data2 + i + 4));             \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 5),              \
                               _mm256_lddqu_si256(data2 + i + 5));             \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 6),              \
                               _mm256_lddqu_si256(data2 + i + 6));             \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 7),              \
                               _mm256_lddqu_si256(data2 + i + 7));             \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursB, &twos, twos, twosA, twosB);                           \
            CSA(&eightsA, &fours, fours, foursA, foursB);                      \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 8),              \
                               _mm256_lddqu_si256(data2 + i + 8));             \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 9),              \
                               _mm256_lddqu_si256(data2 + i + 9));             \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 10),             \
                               _mm256_lddqu_si256(data2 + i + 10));            \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 11),             \
                               _mm256_lddqu_si256(data2 + i + 11));            \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursA, &twos, twos, twosA, twosB);                           \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 12),             \
                               _mm256_lddqu_si256(data2 + i + 12));            \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 13),             \
                               _mm256_lddqu_si256(data2 + i + 13));            \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 14),             \
                               _mm256_lddqu_si256(data2 + i + 14));            \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 15),             \
                               _mm256_lddqu_si256(data2 + i + 15));            \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursB, &twos, twos, twosA, twosB);                           \
            CSA(&eightsB, &fours, fours, foursA, foursB);                      \
            CSA(&sixteens, &eights, eights, eightsA, eightsB);                 \
            total = _mm256_add_epi64(total, popcount256(sixteens));            \
        }                                                                      \
        total = _mm256_slli_epi64(total, 4);                                   \
        total = _mm256_add_epi64(total,                                        \
                                 _mm256_slli_epi64(popcount256(eights), 3));   \
        total =                                                                \
            _mm256_add_epi64(total, _mm256_slli_epi64(popcount256(fours), 2)); \
        total =                                                                \
            _mm256_add_epi64(total, _mm256_slli_epi64(popcount256(twos), 1));  \
        total = _mm256_add_epi64(total, popcount256(ones));                    \
        for (; i < size; i++) {                                                \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i),                  \
                               _mm256_lddqu_si256(data2 + i));                 \
            total = _mm256_add_epi64(total, popcount256(A1));                  \
        }                                                                      \
        return (uint64_t)(_mm256_extract_epi64(total, 0)) +                    \
               (uint64_t)(_mm256_extract_epi64(total, 1)) +                    \
               (uint64_t)(_mm256_extract_epi64(total, 2)) +                    \
               (uint64_t)(_mm256_extract_epi64(total, 3));                     \
    }                                                                          \
    static inline uint64_t avx2_harley_seal_popcount256andstore_##opname(      \
        const __m256i *__restrict__ data1, const __m256i *__restrict__ data2,          \
        __m256i *__restrict__ out, const uint64_t size) {                          \
        __m256i total = _mm256_setzero_si256();                                \
        __m256i ones = _mm256_setzero_si256();                                 \
        __m256i twos = _mm256_setzero_si256();                                 \
        __m256i fours = _mm256_setzero_si256();                                \
        __m256i eights = _mm256_setzero_si256();                               \
        __m256i sixteens = _mm256_setzero_si256();                             \
        __m256i twosA, twosB, foursA, foursB, eightsA, eightsB;                \
        __m256i A1, A2;                                                        \
        const uint64_t limit = size - size % 16;                               \
        uint64_t i = 0;                                                        \
        for (; i < limit; i += 16) {                                           \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i),                  \
                               _mm256_lddqu_si256(data2 + i));                 \
            _mm256_storeu_si256(out + i, A1);                                  \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 1),              \
                               _mm256_lddqu_si256(data2 + i + 1));             \
            _mm256_storeu_si256(out + i + 1, A2);                              \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 2),              \
                               _mm256_lddqu_si256(data2 + i + 2));             \
            _mm256_storeu_si256(out + i + 2, A1);                              \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 3),              \
                               _mm256_lddqu_si256(data2 + i + 3));             \
            _mm256_storeu_si256(out + i + 3, A2);                              \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursA, &twos, twos, twosA, twosB);                           \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 4),              \
                               _mm256_lddqu_si256(data2 + i + 4));             \
            _mm256_storeu_si256(out + i + 4, A1);                              \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 5),              \
                               _mm256_lddqu_si256(data2 + i + 5));             \
            _mm256_storeu_si256(out + i + 5, A2);                              \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 6),              \
                               _mm256_lddqu_si256(data2 + i + 6));             \
            _mm256_storeu_si256(out + i + 6, A1);                              \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 7),              \
                               _mm256_lddqu_si256(data2 + i + 7));             \
            _mm256_storeu_si256(out + i + 7, A2);                              \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursB, &twos, twos, twosA, twosB);                           \
            CSA(&eightsA, &fours, fours, foursA, foursB);                      \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 8),              \
                               _mm256_lddqu_si256(data2 + i + 8));             \
            _mm256_storeu_si256(out + i + 8, A1);                              \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 9),              \
                               _mm256_lddqu_si256(data2 + i + 9));             \
            _mm256_storeu_si256(out + i + 9, A2);                              \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 10),             \
                               _mm256_lddqu_si256(data2 + i + 10));            \
            _mm256_storeu_si256(out + i + 10, A1);                             \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 11),             \
                               _mm256_lddqu_si256(data2 + i + 11));            \
            _mm256_storeu_si256(out + i + 11, A2);                             \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursA, &twos, twos, twosA, twosB);                           \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 12),             \
                               _mm256_lddqu_si256(data2 + i + 12));            \
            _mm256_storeu_si256(out + i + 12, A1);                             \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 13),             \
                               _mm256_lddqu_si256(data2 + i + 13));            \
            _mm256_storeu_si256(out + i + 13, A2);                             \
            CSA(&twosA, &ones, ones, A1, A2);                                  \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 14),             \
                               _mm256_lddqu_si256(data2 + i + 14));            \
            _mm256_storeu_si256(out + i + 14, A1);                             \
            A2 = avx_intrinsic(_mm256_lddqu_si256(data1 + i + 15),             \
                               _mm256_lddqu_si256(data2 + i + 15));            \
            _mm256_storeu_si256(out + i + 15, A2);                             \
            CSA(&twosB, &ones, ones, A1, A2);                                  \
            CSA(&foursB, &twos, twos, twosA, twosB);                           \
            CSA(&eightsB, &fours, fours, foursA, foursB);                      \
            CSA(&sixteens, &eights, eights, eightsA, eightsB);                 \
            total = _mm256_add_epi64(total, popcount256(sixteens));            \
        }                                                                      \
        total = _mm256_slli_epi64(total, 4);                                   \
        total = _mm256_add_epi64(total,                                        \
                                 _mm256_slli_epi64(popcount256(eights), 3));   \
        total =                                                                \
            _mm256_add_epi64(total, _mm256_slli_epi64(popcount256(fours), 2)); \
        total =                                                                \
            _mm256_add_epi64(total, _mm256_slli_epi64(popcount256(twos), 1));  \
        total = _mm256_add_epi64(total, popcount256(ones));                    \
        for (; i < size; i++) {                                                \
            A1 = avx_intrinsic(_mm256_lddqu_si256(data1 + i),                  \
                               _mm256_lddqu_si256(data2 + i));                 \
            _mm256_storeu_si256(out + i, A1);                                  \
            total = _mm256_add_epi64(total, popcount256(A1));                  \
        }                                                                      \
        return (uint64_t)(_mm256_extract_epi64(total, 0)) +                    \
               (uint64_t)(_mm256_extract_epi64(total, 1)) +                    \
               (uint64_t)(_mm256_extract_epi64(total, 2)) +                    \
               (uint64_t)(_mm256_extract_epi64(total, 3));                     \
    }

AVXPOPCNTFNC(or, _mm256_or_si256)
AVXPOPCNTFNC(union, _mm256_or_si256)
AVXPOPCNTFNC(and, _mm256_and_si256)
AVXPOPCNTFNC(intersection, _mm256_and_si256)
AVXPOPCNTFNC (xor, _mm256_xor_si256)
AVXPOPCNTFNC(andnot, _mm256_andnot_si256)

/***
 * END Harley-Seal popcount functions.
 */

#endif  // USEAVX

#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/bitset_util.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/array.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/array.h"
/*
 * array.h
 *
 */

#ifndef INCLUDE_CONTAINERS_ARRAY_H_
#define INCLUDE_CONTAINERS_ARRAY_H_

#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>


/* Containers with DEFAULT_MAX_SIZE or less integers should be arrays */
enum { DEFAULT_MAX_SIZE = 4096 };

/* struct array_container - sparse representation of a bitmap
 *
 * @cardinality: number of indices in `array` (and the bitmap)
 * @capacity:    allocated size of `array`
 * @array:       sorted list of integers
 */
struct array_container_s {
    int32_t cardinality;
    int32_t capacity;
    uint16_t *array;
};

typedef struct array_container_s array_container_t;

/* Create a new array with default. Return NULL in case of failure. See also
 * array_container_create_given_capacity. */
array_container_t *array_container_create(void);

/* Create a new array with a specified capacity size. Return NULL in case of
 * failure. */
array_container_t *array_container_create_given_capacity(int32_t size);

/* Free memory owned by `array'. */
void array_container_free(array_container_t *array);

/* Duplicate container */
array_container_t *array_container_clone(const array_container_t *src);

int32_t array_container_serialize(array_container_t *container,
                                  char *buf) WARN_UNUSED;

uint32_t array_container_serialization_len(array_container_t *container);

void *array_container_deserialize(const char *buf, size_t buf_len);

/* Get the cardinality of `array'. */
static inline int array_container_cardinality(const array_container_t *array) {
    return array->cardinality;
}

static inline bool array_container_nonzero_cardinality(
    const array_container_t *array) {
    return array->cardinality > 0;
}

/* Copy one container into another. We assume that they are distinct. */
void array_container_copy(const array_container_t *src, array_container_t *dst);

/*  Add all the values in [min,max) (included) at a distance k*step from min.
    The container must have a size less or equal to DEFAULT_MAX_SIZE after this
   addition. */
void array_container_add_from_range(array_container_t *arr, uint32_t min,
                                    uint32_t max, uint16_t step);

/* Set the cardinality to zero (does not release memory). */
static inline void array_container_clear(array_container_t *array) {
    array->cardinality = 0;
}

static inline bool array_container_empty(const array_container_t *array) {
    return array->cardinality == 0;
}

static inline bool array_container_full(const array_container_t *array) {
    return array->cardinality == array->capacity;
}

/* Compute the union of `src_1' and `src_2' and write the result to `dst'
 * It is assumed that `dst' is distinct from both `src_1' and `src_2'. */
void array_container_union(const array_container_t *src_1,
                           const array_container_t *src_2,
                           array_container_t *dst);

/* symmetric difference, see array_container_union */
void array_container_xor(const array_container_t *array_1,
                         const array_container_t *array_2,
                         array_container_t *out);

/* Compute the intersection of src_1 and src_2 and write the result to
 * dst. It is assumed that dst is distinct from both src_1 and src_2. */
void array_container_intersection(const array_container_t *src_1,
                                  const array_container_t *src_2,
                                  array_container_t *dst);

/* computes the intersection of array1 and array2 and write the result to
 * array1.
 * */
void array_container_intersection_inplace(array_container_t *src_1,
                                          const array_container_t *src_2);

/* computes the negation of an array container src, writing to dst,
 *  assumed distinct from src
 *  moved to mixed_negation  TODO: clean me up here
void array_container_negation(const array_container_t *src,
                              array_container_t *dst);

 TODO delete me too when mixed is ok
* computes the negation of an array container src_dest, writing to src_dest
 * Requires result fits* /
void array_container_negation_inplace(array_container_t *src_dest);
*/

/*
 * Write out the 16-bit integers contained in this container as a list of 32-bit
 * integers using base
 * as the starting value (it might be expected that base has zeros in its 16
 * least significant bits).
 * The function returns the number of values written.
 * The caller is responsible for allocating enough memory in out.
 */
int array_container_to_uint32_array(void *vout,
                                    const array_container_t *cont,
                                    uint32_t base);

/* Compute the number of runs */
int32_t array_container_number_of_runs(const array_container_t *a);

/*
 * Print this container using printf (useful for debugging).
 */
void array_container_printf(const array_container_t *v);

/*
 * Print this container using printf as a comma-separated list of 32-bit
 * integers starting at base.
 */
void array_container_printf_as_uint32_array(const array_container_t *v,
                                            uint32_t base);

/**
 * Return the serialized size in bytes of a container having cardinality "card".
 */
static inline int32_t array_container_serialized_size_in_bytes(int32_t card) {
    return card * 2 + 2;
}

/**
 * increase capacity to at least min, and to no more than max. Whether the
 * existing data needs to be copied over depends on the value of the "preserve"
 * parameter.
 * If preserve is false,
 * then the new content will be uninitialized, otherwise the original data is
 * copied.
 */
void array_container_grow(array_container_t *container, int32_t min,
                          int32_t max, bool preserve);

bool array_container_iterate(const array_container_t *cont, uint32_t base,
                             roaring_iterator iterator, void *ptr);

/**
 * Writes the underlying array to buf, outputs how many bytes were written.
 * This is meant to be byte-by-byte compatible with the Java and Go versions of
 * Roaring.
 * The number of bytes written should be
 * array_container_size_in_bytes(container).
 *
 */
int32_t array_container_write(const array_container_t *container, char *buf);
/**
 * Reads the instance from buf, outputs how many bytes were read.
 * This is meant to be byte-by-byte compatible with the Java and Go versions of
 * Roaring.
 * The number of bytes read should be array_container_size_in_bytes(container).
 * You need to provide the (known) cardinality.
 */
int32_t array_container_read(int32_t cardinality, array_container_t *container,
                             const char *buf);

/**
 * Return the serialized size in bytes of a container (see
 * bitset_container_write)
 * This is meant to be compatible with the Java and Go versions of Roaring and
 * assumes
 * that the cardinality of the container is already known.
 *
 */
static inline int32_t array_container_size_in_bytes(
    const array_container_t *container) {
    return container->cardinality * sizeof(uint16_t);
}

/**
 * Return true if the two arrays have the same content.
 */
bool array_container_equals(array_container_t *container1,
                            array_container_t *container2);

/**
 * If the element of given rank is in this container, supposing that the first
 * element has rank start_rank, then the function returns true and sets element
 * accordingly.
 * Otherwise, it returns false and update start_rank.
 */
static inline bool array_container_select(const array_container_t *container,
                                          uint32_t *start_rank, uint32_t rank,
                                          uint32_t *element) {
    int card = array_container_cardinality(container);
    if (*start_rank + card <= rank) {
        *start_rank += card;
        return false;
    } else {
        *element = container->array[rank - *start_rank];
        return true;
    }
}

/* Computes the  difference of array1 and array2 and write the result
 * to array out.
 * Array out does not need to be distinct from array_1
 */
void array_container_andnot(const array_container_t *array_1,
                            const array_container_t *array_2,
                            array_container_t *out);

/* Append x to the set. Assumes that the value is larger than any preceding
 * values.  */
static void array_container_append(array_container_t *arr, uint16_t pos) {
    const int32_t capacity = arr->capacity;

    if (array_container_full(arr)) {
        array_container_grow(arr, capacity + 1, INT32_MAX, true);
    }

    arr->array[arr->cardinality++] = pos;
}

/* Add x to the set. Returns true if x was not already present.  */
static inline bool array_container_add(array_container_t *arr, uint16_t pos) {
    const int32_t cardinality = arr->cardinality;

    // best case, we can append.
    if (array_container_empty(arr) || (arr->array[cardinality - 1] < pos)) {
        array_container_append(arr, pos);
        return true;
    }

    const int32_t loc = binarySearch(arr->array, cardinality, pos);
    const bool not_found = loc < 0;

    if (not_found) {
        if (array_container_full(arr)) {
            array_container_grow(arr, arr->capacity + 1, INT32_MAX, true);
        }
        const int32_t insert_idx = -loc - 1;
        memmove(arr->array + insert_idx + 1, arr->array + insert_idx,
                (cardinality - insert_idx) * sizeof(uint16_t));
        arr->array[insert_idx] = pos;
        arr->cardinality++;
    }

    return not_found;
}

/* Remove x from the set. Returns true if x was present.  */
static inline bool array_container_remove(array_container_t *arr,
                                          uint16_t pos) {
    const int32_t idx = binarySearch(arr->array, arr->cardinality, pos);
    const bool is_present = idx >= 0;
    if (is_present) {
        memmove(arr->array + idx, arr->array + idx + 1,
                (arr->cardinality - idx - 1) * sizeof(uint16_t));
        arr->cardinality--;
    }

    return is_present;
}

/* Check whether x is present.  */
inline bool array_container_contains(const array_container_t *arr,
                                            uint16_t pos) {
    return binarySearch(arr->array, arr->cardinality, pos) >= 0;
}

#endif /* INCLUDE_CONTAINERS_ARRAY_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/array.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/bitset.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/bitset.h"
/*
 * bitset.h
 *
 */

#ifndef INCLUDE_CONTAINERS_BITSET_H_
#define INCLUDE_CONTAINERS_BITSET_H_

#include <stdbool.h>
#include <stdint.h>

#ifdef USEAVX
#define ALIGN_AVX __attribute__((aligned(sizeof(__m256i))))
#else
#define ALIGN_AVX
#endif

enum {
    BITSET_CONTAINER_SIZE_IN_WORDS = (1 << 16) / 64,
    BITSET_UNKNOWN_CARDINALITY = -1
};

struct bitset_container_s {
    int32_t cardinality;
    uint64_t *array;
};

typedef struct bitset_container_s bitset_container_t;

/* Create a new bitset. Return NULL in case of failure. */
bitset_container_t *bitset_container_create(void);

/* Free memory. */
void bitset_container_free(bitset_container_t *bitset);

/* Clear bitset (sets bits to 0). */
void bitset_container_clear(bitset_container_t *bitset);

/* Set all bits to 1. */
void bitset_container_set_all(bitset_container_t *bitset);

/* Duplicate bitset */
bitset_container_t *bitset_container_clone(const bitset_container_t *src);

int32_t bitset_container_serialize(bitset_container_t *container,
                                   char *buf) WARN_UNUSED;

uint32_t bitset_container_serialization_len(void);

void *bitset_container_deserialize(const char *buf, size_t buf_len);

/* Set the bit in [begin,end). WARNING: as of April 2016, this method is slow
 * and
 * should not be used in performance-sensitive code. Ever.  */
void bitset_container_set_range(bitset_container_t *bitset, uint32_t begin,
                                uint32_t end);

#ifdef ASMBITMANIPOPTIMIZATION
/* Set the ith bit.  */
static inline void bitset_container_set(bitset_container_t *bitset,
                                        uint16_t pos) {
    uint64_t shift = 6;
    uint64_t offset;
    uint64_t p = pos;
    ASM_SHIFT_RIGHT(p, shift, offset);
    uint64_t load = bitset->array[offset];
    ASM_SET_BIT_INC_WAS_CLEAR(load, p, bitset->cardinality);
    bitset->array[offset] = load;
}

/* Unset the ith bit.  */
static inline void bitset_container_unset(bitset_container_t *bitset,
                                          uint16_t pos) {
    uint64_t shift = 6;
    uint64_t offset;
    uint64_t p = pos;
    ASM_SHIFT_RIGHT(p, shift, offset);
    uint64_t load = bitset->array[offset];
    ASM_CLEAR_BIT_DEC_WAS_SET(load, p, bitset->cardinality);
    bitset->array[offset] = load;
}

/* Add `pos' to `bitset'. Returns true if `pos' was not present. Might be slower
 * than bitset_container_set.  */
static inline bool bitset_container_add(bitset_container_t *bitset,
                                        uint16_t pos) {
    uint64_t shift = 6;
    uint64_t offset;
    uint64_t p = pos;
    ASM_SHIFT_RIGHT(p, shift, offset);
    uint64_t load = bitset->array[offset];
    // could be possibly slightly further optimized
    const int32_t oldcard = bitset->cardinality;
    ASM_SET_BIT_INC_WAS_CLEAR(load, p, bitset->cardinality);
    bitset->array[offset] = load;
    return bitset->cardinality - oldcard;
}

/* Remove `pos' from `bitset'. Returns true if `pos' was present.  Might be
 * slower than bitset_container_unset.  */
static inline bool bitset_container_remove(bitset_container_t *bitset,
                                           uint16_t pos) {
    uint64_t shift = 6;
    uint64_t offset;
    uint64_t p = pos;
    ASM_SHIFT_RIGHT(p, shift, offset);
    uint64_t load = bitset->array[offset];
    // could be possibly slightly further optimized
    const int32_t oldcard = bitset->cardinality;
    ASM_CLEAR_BIT_DEC_WAS_SET(load, p, bitset->cardinality);
    bitset->array[offset] = load;
    return oldcard - bitset->cardinality;
}

/* Get the value of the ith bit.  */
inline bool bitset_container_get(const bitset_container_t *bitset,
                                        uint16_t pos) {
    uint64_t word = bitset->array[pos >> 6];
    const uint64_t p = pos;
    ASM_INPLACESHIFT_RIGHT(word, p);
    return word & 1;
}

#else

/* Set the ith bit.  */
static inline void bitset_container_set(bitset_container_t *bitset,
                                        uint16_t pos) {
    const uint64_t old_word = bitset->array[pos >> 6];
    const int index = pos & 63;
    const uint64_t new_word = old_word | (UINT64_C(1) << index);
    bitset->cardinality += (old_word ^ new_word) >> index;
    bitset->array[pos >> 6] = new_word;
}

/* Unset the ith bit.  */
static inline void bitset_container_unset(bitset_container_t *bitset,
                                          uint16_t pos) {
    const uint64_t old_word = bitset->array[pos >> 6];
    const int index = pos & 63;
    const uint64_t new_word = old_word & (~(UINT64_C(1) << index));
    bitset->cardinality -= (old_word ^ new_word) >> index;
    bitset->array[pos >> 6] = new_word;
}

/* Add `pos' to `bitset'. Returns true if `pos' was not present. Might be slower
 * than bitset_container_set.  */
static inline bool bitset_container_add(bitset_container_t *bitset,
                                        uint16_t pos) {
    const uint64_t old_word = bitset->array[pos >> 6];
    const int index = pos & 63;
    const uint64_t new_word = old_word | (UINT64_C(1) << index);
    const uint64_t increment = (old_word ^ new_word) >> index;
    bitset->cardinality += increment;
    bitset->array[pos >> 6] = new_word;
    return increment;  // 0 == false, 1 == true
}

/* Remove `pos' from `bitset'. Returns true if `pos' was present.  Might be
 * slower than bitset_container_unset.  */
static inline bool bitset_container_remove(bitset_container_t *bitset,
                                           uint16_t pos) {
    const uint64_t old_word = bitset->array[pos >> 6];
    const int index = pos & 63;
    const uint64_t new_word = old_word & (~(UINT64_C(1) << index));
    const uint64_t increment = (old_word ^ new_word) >> index;
    bitset->cardinality -= increment;
    bitset->array[pos >> 6] = new_word;
    return increment;  // 0 == false, 1 == true
}

/* Get the value of the ith bit.  */
static inline bool bitset_container_get(const bitset_container_t *bitset,
                                        uint16_t pos) {
    const uint64_t word = bitset->array[pos >> 6];
    // getting rid of the mask can shave one cycle off...
    return (word >> (pos & 63)) & 1;
}

#endif

/* Check whether `bitset' is present in `array'.  Calls bitset_container_get. */
static inline bool bitset_container_contains(const bitset_container_t *bitset,
                                             uint16_t pos) {
    return bitset_container_get(bitset, pos);
}

/* Get the number of bits set */
static inline int bitset_container_cardinality(
    const bitset_container_t *bitset) {
    return bitset->cardinality;
}

/* Copy one container into another. We assume that they are distinct. */
void bitset_container_copy(const bitset_container_t *source,
                           bitset_container_t *dest);

/*  Add all the values [min,max) at a distance k*step from min: min,
 * min+step,.... */
void bitset_container_add_from_range(bitset_container_t *bitset, uint32_t min,
                                     uint32_t max, uint16_t step);

/* Get the number of bits set (force computation). This does not modify bitset.
 * To update the cardinality, you should do
 * bitset->cardinality =  bitset_container_compute_cardinality(bitset).*/
int bitset_container_compute_cardinality(const bitset_container_t *bitset);

/* Get whether there is at least one bit set  */
static inline bool bitset_container_nonzero_cardinality(
    bitset_container_t *bitset) {
    // account for laziness
    if (bitset->cardinality == BITSET_UNKNOWN_CARDINALITY)
        // could bail early instead with a nonzero result
        bitset->cardinality = bitset_container_compute_cardinality(bitset);
    return bitset->cardinality > 0;
}

/* Computes the union of bitsets `src_1' and `src_2' into `dst'  and return the
 * cardinality. */
int bitset_container_or(const bitset_container_t *src_1,
                        const bitset_container_t *src_2,
                        bitset_container_t *dst);

/* Computes the union of bitsets `src_1' and `src_2' and return the cardinality.
 */
int bitset_container_or_justcard(const bitset_container_t *src_1,
                                 const bitset_container_t *src_2);

/* Computes the union of bitsets `src_1' and `src_2' into `dst' and return the
 * cardinality. Same as bitset_container_or. */
int bitset_container_union(const bitset_container_t *src_1,
                           const bitset_container_t *src_2,
                           bitset_container_t *dst);

/* Computes the union of bitsets `src_1' and `src_2'  and return the
 * cardinality. Same as bitset_container_or_justcard. */
int bitset_container_union_justcard(const bitset_container_t *src_1,
                                    const bitset_container_t *src_2);

/* Computes the union of bitsets `src_1' and `src_2' into `dst', but does not
 * update the cardinality. Provided to optimize chained operations. */
int bitset_container_or_nocard(const bitset_container_t *src_1,
                               const bitset_container_t *src_2,
                               bitset_container_t *dst);

/* Computes the intersection of bitsets `src_1' and `src_2' into `dst' and
 * return the cardinality. */
int bitset_container_and(const bitset_container_t *src_1,
                         const bitset_container_t *src_2,
                         bitset_container_t *dst);

/* Computes the intersection of bitsets `src_1' and `src_2'  and return the
 * cardinality. */
int bitset_container_and_justcard(const bitset_container_t *src_1,
                                  const bitset_container_t *src_2);

/* Computes the intersection of bitsets `src_1' and `src_2' into `dst' and
 * return the cardinality. Same as bitset_container_and. */
int bitset_container_intersection(const bitset_container_t *src_1,
                                  const bitset_container_t *src_2,
                                  bitset_container_t *dst);

/* Computes the intersection of bitsets `src_1' and `src_2' and return the
 * cardinality. Same as bitset_container_and_justcard. */
int bitset_container_intersection_justcard(const bitset_container_t *src_1,
                                           const bitset_container_t *src_2);

/* Computes the intersection of bitsets `src_1' and `src_2' into `dst', but does
 * not update the cardinality. Provided to optimize chained operations. */
int bitset_container_and_nocard(const bitset_container_t *src_1,
                                const bitset_container_t *src_2,
                                bitset_container_t *dst);

/* Computes the exclusive or of bitsets `src_1' and `src_2' into `dst' and
 * return the cardinality. */
int bitset_container_xor(const bitset_container_t *src_1,
                         const bitset_container_t *src_2,
                         bitset_container_t *dst);

/* Computes the exclusive or of bitsets `src_1' and `src_2' and return the
 * cardinality. */
int bitset_container_xor_justcard(const bitset_container_t *src_1,
                                  const bitset_container_t *src_2);

/* Computes the exclusive or of bitsets `src_1' and `src_2' into `dst', but does
 * not update the cardinality. Provided to optimize chained operations. */
int bitset_container_xor_nocard(const bitset_container_t *src_1,
                                const bitset_container_t *src_2,
                                bitset_container_t *dst);

/* Computes the and not of bitsets `src_1' and `src_2' into `dst' and return the
 * cardinality. */
int bitset_container_andnot(const bitset_container_t *src_1,
                            const bitset_container_t *src_2,
                            bitset_container_t *dst);

/* Computes the and not of bitsets `src_1' and `src_2'  and return the
 * cardinality. */
int bitset_container_andnot_justcard(const bitset_container_t *src_1,
                                     const bitset_container_t *src_2);

/* Computes the and not or of bitsets `src_1' and `src_2' into `dst', but does
 * not update the cardinality. Provided to optimize chained operations. */
int bitset_container_andnot_nocard(const bitset_container_t *src_1,
                                   const bitset_container_t *src_2,
                                   bitset_container_t *dst);

/*
 * Write out the 16-bit integers contained in this container as a list of 32-bit
 * integers using base
 * as the starting value (it might be expected that base has zeros in its 16
 * least significant bits).
 * The function returns the number of values written.
 * The caller is responsible for allocating enough memory in out.
 * The out pointer should point to enough memory (the cardinality times 32
 * bits).
 */
int bitset_container_to_uint32_array(void *out,
                                     const bitset_container_t *cont,
                                     uint32_t base);

/*
 * Print this container using printf (useful for debugging).
 */
void bitset_container_printf(const bitset_container_t *v);

/*
 * Print this container using printf as a comma-separated list of 32-bit
 * integers starting at base.
 */
void bitset_container_printf_as_uint32_array(const bitset_container_t *v,
                                             uint32_t base);

/**
 * Return the serialized size in bytes of a container.
 */
static inline int32_t bitset_container_serialized_size_in_bytes(void) {
    return BITSET_CONTAINER_SIZE_IN_WORDS * 8;
}

/**
 * Return the the number of runs.
 */
int bitset_container_number_of_runs(bitset_container_t *b);

bool bitset_container_iterate(const bitset_container_t *cont, uint32_t base,
                              roaring_iterator iterator, void *ptr);

/**
 * Writes the underlying array to buf, outputs how many bytes were written.
 * This is meant to be byte-by-byte compatible with the Java and Go versions of
 * Roaring.
 * The number of bytes written should be
 * bitset_container_size_in_bytes(container).
 */
int32_t bitset_container_write(const bitset_container_t *container, char *buf);

/**
 * Reads the instance from buf, outputs how many bytes were read.
 * This is meant to be byte-by-byte compatible with the Java and Go versions of
 * Roaring.
 * The number of bytes read should be bitset_container_size_in_bytes(container).
 * You need to provide the (known) cardinality.
 */
int32_t bitset_container_read(int32_t cardinality,
                              bitset_container_t *container, const char *buf);
/**
 * Return the serialized size in bytes of a container (see
 * bitset_container_write).
 * This is meant to be compatible with the Java and Go versions of Roaring and
 * assumes
 * that the cardinality of the container is already known or can be computed.
 */
static inline int32_t bitset_container_size_in_bytes(
    const bitset_container_t *container) {
    (void)container;
    return BITSET_CONTAINER_SIZE_IN_WORDS * sizeof(uint64_t);
}

/**
 * Return true if the two containers have the same content.
 */
bool bitset_container_equals(bitset_container_t *container1,
                             bitset_container_t *container2);

/**
 * If the element of given rank is in this container, supposing that the first
 * element has rank start_rank, then the function returns true and sets element
 * accordingly.
 * Otherwise, it returns false and update start_rank.
 */
bool bitset_container_select(const bitset_container_t *container,
                             uint32_t *start_rank, uint32_t rank,
                             uint32_t *element);

#endif /* INCLUDE_CONTAINERS_BITSET_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/bitset.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/run.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/run.h"
/*
 * run.h
 *
 */

#ifndef INCLUDE_CONTAINERS_RUN_H_
#define INCLUDE_CONTAINERS_RUN_H_

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>
#include <string.h>


/* struct rle16_s - run length pair
 *
 * @value:  start position of the run
 * @length: length of the run is `length + 1`
 *
 * An RLE pair {v, l} would represent the integers between the interval
 * [v, v+l+1], e.g. {3, 2} = [3, 4, 5].
 */
struct rle16_s {
    uint16_t value;
    uint16_t length;
};

typedef struct rle16_s rle16_t;

/* struct run_container_s - run container bitmap
 *
 * @n_runs:   number of rle_t pairs in `runs`.
 * @capacity: capacity in rle_t pairs `runs` can hold.
 * @runs:     pairs of rle_t.
 *
 */
struct run_container_s {
    int32_t n_runs;
    int32_t capacity;
    rle16_t *runs;
};

typedef struct run_container_s run_container_t;

/* Create a new run container. Return NULL in case of failure. */
run_container_t *run_container_create(void);

/* Create a new run container with given capacity. Return NULL in case of
 * failure. */
run_container_t *run_container_create_given_capacity(int32_t size);

/* Free memory owned by `run'. */
void run_container_free(run_container_t *run);

/* Duplicate container */
run_container_t *run_container_clone(const run_container_t *src);

int32_t run_container_serialize(run_container_t *container,
                                char *buf) WARN_UNUSED;

uint32_t run_container_serialization_len(run_container_t *container);

void *run_container_deserialize(const char *buf, size_t buf_len);

/*
 * Effectively deletes the value at index index, repacking data.
 */
static void recoverRoomAtIndex(run_container_t *run, uint16_t index) {
    memmove(run->runs + index, run->runs + (1 + index),
            (run->n_runs - index - 1) * sizeof(rle16_t));
    run->n_runs--;
}

/**
 * Good old binary search through rle data
 */
inline int32_t interleavedBinarySearch(const rle16_t *array,
                                              int32_t lenarray, uint16_t ikey) {
    int32_t low = 0;
    int32_t high = lenarray - 1;
    while (low <= high) {
        int32_t middleIndex = (low + high) >> 1;
        uint16_t middleValue = array[middleIndex].value;
        if (middleValue < ikey) {
            low = middleIndex + 1;
        } else if (middleValue > ikey) {
            high = middleIndex - 1;
        } else {
            return middleIndex;
        }
    }
    return -(low + 1);
}

/**
 * increase capacity to at least min. Whether the
 * existing data needs to be copied over depends on copy. If "copy" is false,
 * then the new content will be uninitialized, otherwise a copy is made.
 */
void run_container_grow(run_container_t *run, int32_t min, bool copy);

/**
 * Moves the data so that we can write data at index
 */
static inline void makeRoomAtIndex(run_container_t *run, uint16_t index) {
    /* This function calls realloc + memmove sequentially to move by one index.
     * Potentially copying twice the array.
     */
    if (run->n_runs + 1 > run->capacity)
        run_container_grow(run, run->n_runs + 1, true);
    memmove(run->runs + 1 + index, run->runs + index,
            (run->n_runs - index) * sizeof(rle16_t));
    run->n_runs++;
}

/* Add `pos' to `run'. Returns true if `pos' was not present. */
bool run_container_add(run_container_t *run, uint16_t pos);

/* Remove `pos' from `run'. Returns true if `pos' was present. */
static inline bool run_container_remove(run_container_t *run, uint16_t pos) {
    int32_t index = interleavedBinarySearch(run->runs, run->n_runs, pos);
    if (index >= 0) {
        int32_t le = run->runs[index].length;
        if (le == 0) {
            recoverRoomAtIndex(run, index);
        } else {
            run->runs[index].value++;
            run->runs[index].length--;
        }
        return true;
    }
    index = -index - 2;  // points to preceding value, possibly -1
    if (index >= 0) {    // possible match
        int32_t offset = pos - run->runs[index].value;
        int32_t le = run->runs[index].length;
        if (offset < le) {
            // need to break in two
            run->runs[index].length = offset - 1;
            // need to insert
            uint16_t newvalue = pos + 1;
            int32_t newlength = le - offset - 1;
            makeRoomAtIndex(run, index + 1);
            run->runs[index + 1].value = newvalue;
            run->runs[index + 1].length = newlength;
            return true;

        } else if (offset == le) {
            run->runs[index].length--;
            return true;
        }
    }
    // no match
    return false;
}

/* Check whether `pos' is present in `run'.  */
inline bool run_container_contains(const run_container_t *run,
                                          uint16_t pos) {
    int32_t index = interleavedBinarySearch(run->runs, run->n_runs, pos);
    if (index >= 0) return true;
    index = -index - 2;  // points to preceding value, possibly -1
    if (index != -1) {   // possible match
        int32_t offset = pos - run->runs[index].value;
        int32_t le = run->runs[index].length;
        if (offset <= le) return true;
    }
    return false;
}

/* Get the cardinality of `run'. Requires an actual computation. */
int run_container_cardinality(const run_container_t *run);

/* Card > 0? */
static inline bool run_container_nonzero_cardinality(
    const run_container_t *run) {
    return run->n_runs > 0;  // runs never empty
}

/* Copy one container into another. We assume that they are distinct. */
void run_container_copy(const run_container_t *src, run_container_t *dst);

/* Set the cardinality to zero (does not release memory). */
static inline void run_container_clear(run_container_t *run) {
    run->n_runs = 0;
}

/**
 * Append run described by vl to the run container, possibly merging.
 * It is assumed that the run would be inserted at the end of the container, no
 * check is made.
 * It is assumed that the run container has the necessary capacity: caller is
 * responsible for checking memory capacity.
 *
 *
 * This is not a safe function, it is meant for performance: use with care.
 */
static inline void run_container_append(run_container_t *run, rle16_t vl,
                                        rle16_t *previousrl) {
    const uint32_t previousend = previousrl->value + previousrl->length;
    if (vl.value > previousend + 1) {  // we add a new one
        run->runs[run->n_runs] = vl;
        run->n_runs++;
        *previousrl = vl;
    } else {
        uint32_t newend = vl.value + vl.length + UINT32_C(1);
        if (newend > previousend) {  // we merge
            previousrl->length = newend - 1 - previousrl->value;
            run->runs[run->n_runs - 1] = *previousrl;
        }
    }
}

/**
 * Like run_container_append but it is assumed that the content of run is empty.
 */
static inline rle16_t run_container_append_first(run_container_t *run,
                                                 rle16_t vl) {
    run->runs[run->n_runs] = vl;
    run->n_runs++;
    return vl;
}

/**
 * append a single value  given by val to the run container, possibly merging.
 * It is assumed that the value would be inserted at the end of the container,
 * no check is made.
 * It is assumed that the run container has the necessary capacity: caller is
 * responsible for checking memory capacity.
 *
 * This is not a safe function, it is meant for performance: use with care.
 */
static inline void run_container_append_value(run_container_t *run,
                                              uint16_t val,
                                              rle16_t *previousrl) {
    const uint32_t previousend = previousrl->value + previousrl->length;
    if (val > previousend + 1) {  // we add a new one
        //*previousrl = (rle16_t){.value = val, .length = 0};// requires C99
        previousrl->value = val;
        previousrl->length = 0;

        run->runs[run->n_runs] = *previousrl;
        run->n_runs++;
    } else if (val == previousend + 1) {  // we merge
        previousrl->length++;
        run->runs[run->n_runs - 1] = *previousrl;
    }
}

/**
 * Like run_container_append_value but it is assumed that the content of run is
 * empty.
 */
static inline rle16_t run_container_append_value_first(run_container_t *run,
                                                       uint16_t val) {
    // rle16_t newrle = (rle16_t){.value = val, .length = 0};// requires C99
    rle16_t newrle;
    newrle.value = val;
    newrle.length = 0;

    run->runs[run->n_runs] = newrle;
    run->n_runs++;
    return newrle;
}

/* Check whether the container spans the whole chunk (cardinality = 1<<16).
 * This check can be done in constant time (inexpensive). */
static inline bool run_container_is_full(const run_container_t *run) {
    rle16_t vl = run->runs[0];
    return (run->n_runs == 1) && (vl.value == 0) && (vl.length == 0xFFFF);
}

/* Compute the union of `src_1' and `src_2' and write the result to `dst'
 * It is assumed that `dst' is distinct from both `src_1' and `src_2'. */
void run_container_union(const run_container_t *src_1,
                         const run_container_t *src_2, run_container_t *dst);

/* Compute the union of `src_1' and `src_2' and write the result to `src_1' */
void run_container_union_inplace(run_container_t *src_1,
                                 const run_container_t *src_2);

/* Compute the intersection of src_1 and src_2 and write the result to
 * dst. It is assumed that dst is distinct from both src_1 and src_2. */
void run_container_intersection(const run_container_t *src_1,
                                const run_container_t *src_2,
                                run_container_t *dst);

/* Compute the symmetric difference of `src_1' and `src_2' and write the result
 * to `dst'
 * It is assumed that `dst' is distinct from both `src_1' and `src_2'. */
void run_container_xor(const run_container_t *src_1,
                       const run_container_t *src_2, run_container_t *dst);

/*
 * Write out the 16-bit integers contained in this container as a list of 32-bit
 * integers using base
 * as the starting value (it might be expected that base has zeros in its 16
 * least significant bits).
 * The function returns the number of values written.
 * The caller is responsible for allocating enough memory in out.
 */
int run_container_to_uint32_array(void *vout, const run_container_t *cont,
                                  uint32_t base);

/*
 * Print this container using printf (useful for debugging).
 */
void run_container_printf(const run_container_t *v);

/*
 * Print this container using printf as a comma-separated list of 32-bit
 * integers starting at base.
 */
void run_container_printf_as_uint32_array(const run_container_t *v,
                                          uint32_t base);

/**
 * Return the serialized size in bytes of a container having "num_runs" runs.
 */
static inline int32_t run_container_serialized_size_in_bytes(int32_t num_runs) {
    return sizeof(uint16_t) +
           sizeof(rle16_t) * num_runs;  // each run requires 2 2-byte entries.
}

bool run_container_iterate(const run_container_t *cont, uint32_t base,
                           roaring_iterator iterator, void *ptr);

/**
 * Writes the underlying array to buf, outputs how many bytes were written.
 * This is meant to be byte-by-byte compatible with the Java and Go versions of
 * Roaring.
 * The number of bytes written should be run_container_size_in_bytes(container).
 */
int32_t run_container_write(const run_container_t *container, char *buf);

/**
 * Reads the instance from buf, outputs how many bytes were read.
 * This is meant to be byte-by-byte compatible with the Java and Go versions of
 * Roaring.
 * The number of bytes read should be bitset_container_size_in_bytes(container).
 * The cardinality parameter is provided for consistency with other containers,
 * but
 * it might be effectively ignored..
 */
int32_t run_container_read(int32_t cardinality, run_container_t *container,
                           const char *buf);

/**
 * Return the serialized size in bytes of a container (see run_container_write).
 * This is meant to be compatible with the Java and Go versions of Roaring.
 */
static inline int32_t run_container_size_in_bytes(
    const run_container_t *container) {
    return run_container_serialized_size_in_bytes(container->n_runs);
}

/**
 * Return true if the two containers have the same content.
 */
bool run_container_equals(run_container_t *container1,
                          run_container_t *container2);

/**
 * Used in a start-finish scan that appends segments, for XOR and NOT
 */

void run_container_smart_append_exclusive(run_container_t *src,
                                          const uint16_t start,
                                          const uint16_t length);

/* The new container consists of a single run. Returns NULL on failure */
static inline run_container_t *run_container_create_range(uint32_t start,
                                                          uint32_t stop) {
    run_container_t *rc = run_container_create_given_capacity(1);
    if (rc) {
        rle16_t r;
        r.value = (uint16_t)start;
        r.length = (uint16_t)(stop - start - 1);
        run_container_append_first(rc, r);
    }
    return rc;
}

/**
 * If the element of given rank is in this container, supposing that the first
 * element has rank start_rank, then the function returns true and sets element
 * accordingly.
 * Otherwise, it returns false and update start_rank.
 */
bool run_container_select(const run_container_t *container,
                          uint32_t *start_rank, uint32_t rank,
                          uint32_t *element);

/* Compute the difference of src_1 and src_2 and write the result to
 * dst. It is assumed that dst is distinct from both src_1 and src_2. */

void run_container_andnot(const run_container_t *src_1,
                          const run_container_t *src_2, run_container_t *dst);

#endif /* INCLUDE_CONTAINERS_RUN_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/run.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/convert.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/convert.h"
/*
 * convert.h
 *
 */

#ifndef INCLUDE_CONTAINERS_CONVERT_H_
#define INCLUDE_CONTAINERS_CONVERT_H_


/* Convert an array into a bitset. The input container is not freed or modified.
 */
bitset_container_t *bitset_container_from_array(const array_container_t *arr);

/* Convert a run into a bitset. The input container is not freed or modified. */
bitset_container_t *bitset_container_from_run(const run_container_t *arr);

/* Convert a run into an array. The input container is not freed or modified. */
array_container_t *array_container_from_run(const run_container_t *arr);

/* Convert a bitset into an array. The input container is not freed or modified.
 */
array_container_t *array_container_from_bitset(const bitset_container_t *bits);

/* Convert an array into a run. The input container is not freed or modified.
 */
run_container_t *run_container_from_array(const array_container_t *c);

/* convert a run into either an array or a bitset
 * might free the container */
void *convert_to_bitset_or_array_container(run_container_t *r, int32_t card,
                                           uint8_t *resulttype);

/* convert containers to and from runcontainers, as is most space efficient.
 * The container might be freed. */
void *convert_run_optimize(void *c, uint8_t typecode_original,
                           uint8_t *typecode_after);

/* converts a run container to either an array or a bitset, IF it saves space.
 */
/* If a conversion occurs, the caller is responsible to free the original
 * container and
 * he becomes reponsible to free the new one. */
void *convert_run_to_efficient_container(run_container_t *c,
                                         uint8_t *typecode_after);
// like convert_run_to_efficient_container but frees the old result if needed
void *convert_run_to_efficient_container_and_free(run_container_t *c,
                                                  uint8_t *typecode_after);
#endif /* INCLUDE_CONTAINERS_CONVERT_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/convert.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_equal.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_equal.h"
/*
 * mixed_equal.h
 *
 */

#ifndef CONTAINERS_MIXED_EQUAL_H_
#define CONTAINERS_MIXED_EQUAL_H_


/**
 * Return true if the two containers have the same content.
 */
bool array_container_equal_bitset(array_container_t* container1,
                                  bitset_container_t* container2);

/**
 * Return true if the two containers have the same content.
 */
bool run_container_equals_array(run_container_t* container1,
                                array_container_t* container2);
/**
 * Return true if the two containers have the same content.
 */
bool run_container_equals_bitset(run_container_t* container1,
                                 bitset_container_t* container2);

#endif /* CONTAINERS_MIXED_EQUAL_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_equal.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_andnot.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_andnot.h"
/*
 * mixed_andnot.h
 */
#ifndef INCLUDE_CONTAINERS_MIXED_ANDNOT_H_
#define INCLUDE_CONTAINERS_MIXED_ANDNOT_H_


/* Compute the andnot of src_1 and src_2 and write the result to
 * dst, a valid array container that could be the same as dst.*/
void array_bitset_container_andnot(const array_container_t *src_1,
                                   const bitset_container_t *src_2,
                                   array_container_t *dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * src_1 */

void array_bitset_container_iandnot(array_container_t *src_1,
                                    const bitset_container_t *src_2);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst, which does not initially have a valid container.
 * Return true for a bitset result; false for array
 */

bool bitset_array_container_andnot(const bitset_container_t *src_1,
                                   const array_container_t *src_2, void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst (which has no container initially).  It will modify src_1
 * to be dst if the result is a bitset.  Otherwise, it will
 * free src_1 and dst will be a new array container.  In both
 * cases, the caller is responsible for deallocating dst.
 * Returns true iff dst is a bitset  */

bool bitset_array_container_iandnot(bitset_container_t *src_1,
                                    const array_container_t *src_2, void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst. Result may be either a bitset or an array container
 * (returns "result is bitset"). dst does not initially have
 * any container, but becomes either a bitset container (return
 * result true) or an array container.
 */

bool run_bitset_container_andnot(const run_container_t *src_1,
                                 const bitset_container_t *src_2, void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst. Result may be either a bitset or an array container
 * (returns "result is bitset"). dst does not initially have
 * any container, but becomes either a bitset container (return
 * result true) or an array container.
 */

bool run_bitset_container_iandnot(run_container_t *src_1,
                                  const bitset_container_t *src_2, void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst. Result may be either a bitset or an array container
 * (returns "result is bitset").  dst does not initially have
 * any container, but becomes either a bitset container (return
 * result true) or an array container.
 */

bool bitset_run_container_andnot(const bitset_container_t *src_1,
                                 const run_container_t *src_2, void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst (which has no container initially).  It will modify src_1
 * to be dst if the result is a bitset.  Otherwise, it will
 * free src_1 and dst will be a new array container.  In both
 * cases, the caller is responsible for deallocating dst.
 * Returns true iff dst is a bitset  */

bool bitset_run_container_iandnot(bitset_container_t *src_1,
                                  const run_container_t *src_2, void **dst);

/* dst does not indicate a valid container initially.  Eventually it
 * can become any type of container.
 */

int run_array_container_andnot(const run_container_t *src_1,
                               const array_container_t *src_2, void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst (which has no container initially).  It will modify src_1
 * to be dst if the result is a bitset.  Otherwise, it will
 * free src_1 and dst will be a new array container.  In both
 * cases, the caller is responsible for deallocating dst.
 * Returns true iff dst is a bitset  */

int run_array_container_iandnot(run_container_t *src_1,
                                const array_container_t *src_2, void **dst);

/* dst must be a valid array container, allowed to be src_1 */

void array_run_container_andnot(const array_container_t *src_1,
                                const run_container_t *src_2,
                                array_container_t *dst);

/* dst does not indicate a valid container initially.  Eventually it
 * can become any kind of container.
 */

void array_run_container_iandnot(array_container_t *src_1,
                                 const run_container_t *src_2);

/* dst does not indicate a valid container initially.  Eventually it
 * can become any kind of container.
 */

int run_run_container_andnot(const run_container_t *src_1,
                             const run_container_t *src_2, void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst (which has no container initially).  It will modify src_1
 * to be dst if the result is a bitset.  Otherwise, it will
 * free src_1 and dst will be a new array container.  In both
 * cases, the caller is responsible for deallocating dst.
 * Returns true iff dst is a bitset  */

int run_run_container_iandnot(run_container_t *src_1,
                              const run_container_t *src_2, void **dst);

/*
 * dst is a valid array container and may be the same as src_1
 */

void array_array_container_andnot(const array_container_t *src_1,
                                  const array_container_t *src_2,
                                  array_container_t *dst);

/* inplace array-array andnot will always be able to reuse the space of
 * src_1 */
void array_array_container_iandnot(array_container_t *src_1,
                                   const array_container_t *src_2);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst (which has no container initially). Return value is
 * "dst is a bitset"
 */

bool bitset_bitset_container_andnot(const bitset_container_t *src_1,
                                    const bitset_container_t *src_2,
                                    void **dst);

/* Compute the andnot of src_1 and src_2 and write the result to
 * dst (which has no container initially).  It will modify src_1
 * to be dst if the result is a bitset.  Otherwise, it will
 * free src_1 and dst will be a new array container.  In both
 * cases, the caller is responsible for deallocating dst.
 * Returns true iff dst is a bitset  */

bool bitset_bitset_container_iandnot(bitset_container_t *src_1,
                                     const bitset_container_t *src_2,
                                     void **dst);
#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_andnot.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_intersection.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_intersection.h"
/*
 * mixed_intersection.h
 *
 */

#ifndef INCLUDE_CONTAINERS_MIXED_INTERSECTION_H_
#define INCLUDE_CONTAINERS_MIXED_INTERSECTION_H_

/* These functions appear to exclude cases where the
 * inputs have the same type and the output is guaranteed
 * to have the same type as the inputs.  Eg, array intersection
 */


/* Compute the intersection of src_1 and src_2 and write the result to
 * dst. It is allowed for dst to be equal to src_1. We assume that dst is a
 * valid container. */
void array_bitset_container_intersection(const array_container_t *src_1,
                                         const bitset_container_t *src_2,
                                         array_container_t *dst);

/*
 * Compute the intersection between src_1 and src_2 and write the result
 * to *dst. If the return function is true, the result is a bitset_container_t
 * otherwise is a array_container_t. We assume that dst is not pre-allocated. In
 * case of failure, *dst will be NULL.
 */
bool bitset_bitset_container_intersection(const bitset_container_t *src_1,
                                          const bitset_container_t *src_2,
                                          void **dst);

/* Compute the intersection of src_1 and src_2 and write the result to
 * dst. It is allowed for dst to be equal to src_1. We assume that dst is a
 * valid container. */
void array_run_container_intersection(const array_container_t *src_1,
                                      const run_container_t *src_2,
                                      array_container_t *dst);

/* Compute the intersection of src_1 and src_2 and write the result to
 * *dst. If the result is true then the result is a bitset_container_t
 * otherwise is a array_container_t.
 * If *dst == src_2, then an in-place intersection is attempted
 **/
bool run_bitset_container_intersection(const run_container_t *src_1,
                                       const bitset_container_t *src_2,
                                       void **dst);

/*
 * Same as bitset_bitset_container_intersection except that if the output is to
 * be a
 * bitset_container_t, then src_1 is modified and no allocation is made.
 * If the output is to be an array_container_t, then caller is responsible
 * to free the container.
 * In all cases, the result is in *dst.
 */
bool bitset_bitset_container_intersection_inplace(
    bitset_container_t *src_1, const bitset_container_t *src_2, void **dst);

#endif /* INCLUDE_CONTAINERS_MIXED_INTERSECTION_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_intersection.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_negation.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_negation.h"
/*
 * mixed_negation.h
 *
 */

#ifndef INCLUDE_CONTAINERS_MIXED_NEGATION_H_
#define INCLUDE_CONTAINERS_MIXED_NEGATION_H_


/* Negation across the entire range of the container.
 * Compute the  negation of src  and write the result
 * to *dst. The complement of a
 * sufficiently sparse set will always be dense and a hence a bitmap
 * We assume that dst is pre-allocated and a valid bitset container
 * There can be no in-place version.
 */
void array_container_negation(const array_container_t *src,
                              bitset_container_t *dst);

/* Negation across the entire range of the container
 * Compute the  negation of src  and write the result
 * to *dst.  A true return value indicates a bitset result,
 * otherwise the result is an array container.
 *  We assume that dst is not pre-allocated. In
 * case of failure, *dst will be NULL.
 */
bool bitset_container_negation(const bitset_container_t *src, void **dst);

/* inplace version */
/*
 * Same as bitset_container_negation except that if the output is to
 * be a
 * bitset_container_t, then src is modified and no allocation is made.
 * If the output is to be an array_container_t, then caller is responsible
 * to free the container.
 * In all cases, the result is in *dst.
 */
bool bitset_container_negation_inplace(bitset_container_t *src, void **dst);

/* Negation across the entire range of container
 * Compute the  negation of src  and write the result
 * to *dst.
 * Return values are the *_TYPECODES as defined * in containers.h
 *  We assume that dst is not pre-allocated. In
 * case of failure, *dst will be NULL.
 */
int run_container_negation(const run_container_t *src, void **dst);

/*
 * Same as run_container_negation except that if the output is to
 * be a
 * run_container_t, and has the capacity to hold the result,
 * then src is modified and no allocation is made.
 * In all cases, the result is in *dst.
 */
int run_container_negation_inplace(run_container_t *src, void **dst);

/* Negation across a range of the container.
 * Compute the  negation of src  and write the result
 * to *dst. Returns true if the result is a bitset container
 * and false for an array container.  *dst is not preallocated.
 */
bool array_container_negation_range(const array_container_t *src,
                                    const int range_start, const int range_end,
                                    void **dst);

/* Even when the result would fit, it is unclear how to make an
 * inplace version without inefficient copying.  Thus this routine
 * may be a wrapper for the non-in-place version
 */
bool array_container_negation_range_inplace(array_container_t *src,
                                            const int range_start,
                                            const int range_end, void **dst);

/* Negation across a range of the container
 * Compute the  negation of src  and write the result
 * to *dst.  A true return value indicates a bitset result,
 * otherwise the result is an array container.
 *  We assume that dst is not pre-allocated. In
 * case of failure, *dst will be NULL.
 */
bool bitset_container_negation_range(const bitset_container_t *src,
                                     const int range_start, const int range_end,
                                     void **dst);

/* inplace version */
/*
 * Same as bitset_container_negation except that if the output is to
 * be a
 * bitset_container_t, then src is modified and no allocation is made.
 * If the output is to be an array_container_t, then caller is responsible
 * to free the container.
 * In all cases, the result is in *dst.
 */
bool bitset_container_negation_range_inplace(bitset_container_t *src,
                                             const int range_start,
                                             const int range_end, void **dst);

/* Negation across a range of container
 * Compute the  negation of src  and write the result
 * to *dst.  Return values are the *_TYPECODES as defined * in containers.h
 *  We assume that dst is not pre-allocated. In
 * case of failure, *dst will be NULL.
 */
int run_container_negation_range(const run_container_t *src,
                                 const int range_start, const int range_end,
                                 void **dst);

/*
 * Same as run_container_negation except that if the output is to
 * be a
 * run_container_t, and has the capacity to hold the result,
 * then src is modified and no allocation is made.
 * In all cases, the result is in *dst.
 */
int run_container_negation_range_inplace(run_container_t *src,
                                         const int range_start,
                                         const int range_end, void **dst);

#endif /* INCLUDE_CONTAINERS_MIXED_NEGATION_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_negation.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_union.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_union.h"
/*
 * mixed_intersection.h
 *
 */

#ifndef INCLUDE_CONTAINERS_MIXED_UNION_H_
#define INCLUDE_CONTAINERS_MIXED_UNION_H_

/* These functions appear to exclude cases where the
 * inputs have the same type and the output is guaranteed
 * to have the same type as the inputs.  Eg, bitset unions
 */


/* Compute the union of src_1 and src_2 and write the result to
 * dst. It is allowed for src_2 to be dst.   */
void array_bitset_container_union(const array_container_t *src_1,
                                  const bitset_container_t *src_2,
                                  bitset_container_t *dst);

/* Compute the union of src_1 and src_2 and write the result to
 * dst. It is allowed for src_2 to be dst.  This version does not
 * update the cardinality of dst (it is set to BITSET_UNKNOWN_CARDINALITY). */
void array_bitset_container_lazy_union(const array_container_t *src_1,
                                       const bitset_container_t *src_2,
                                       bitset_container_t *dst);

/*
 * Compute the union between src_1 and src_2 and write the result
 * to *dst. If the return function is true, the result is a bitset_container_t
 * otherwise is a array_container_t. We assume that dst is not pre-allocated. In
 * case of failure, *dst will be NULL.
 */
bool array_array_container_union(const array_container_t *src_1,
                                 const array_container_t *src_2, void **dst);

/*
 * Same as array_array_container_union except that it will more eagerly produce
 * a bitset.
 */
bool array_array_container_lazy_union(const array_container_t *src_1,
                                      const array_container_t *src_2,
                                      void **dst);

/* Compute the union of src_1 and src_2 and write the result to
 * dst. We assume that dst is a
 * valid container. The result might need to be further converted to array or
 * bitset container,
 * the caller is responsible for the eventual conversion. */
void array_run_container_union(const array_container_t *src_1,
                               const run_container_t *src_2,
                               run_container_t *dst);

/* Compute the union of src_1 and src_2 and write the result to
 * src2. The result might need to be further converted to array or
 * bitset container,
 * the caller is responsible for the eventual conversion. */
void array_run_container_inplace_union(const array_container_t *src_1,
                                       run_container_t *src_2);

/* Compute the union of src_1 and src_2 and write the result to
 * dst. It is allowed for dst to be src_2.  */
void run_bitset_container_union(const run_container_t *src_1,
                                const bitset_container_t *src_2,
                                bitset_container_t *dst);

/* Compute the union of src_1 and src_2 and write the result to
 * dst. It is allowed for dst to be src_2.  This version does not
 * update the cardinality of dst (it is set to BITSET_UNKNOWN_CARDINALITY). */
void run_bitset_container_lazy_union(const run_container_t *src_1,
                                     const bitset_container_t *src_2,
                                     bitset_container_t *dst);

#endif /* INCLUDE_CONTAINERS_MIXED_UNION_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_union.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_xor.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_xor.h"
/*
 * mixed_xor.h
 *
 */

#ifndef INCLUDE_CONTAINERS_MIXED_XOR_H_
#define INCLUDE_CONTAINERS_MIXED_XOR_H_

/* These functions appear to exclude cases where the
 * inputs have the same type and the output is guaranteed
 * to have the same type as the inputs.  Eg, bitset unions
 */

/*
 * Java implementation (as of May 2016) for array_run, run_run
 * and  bitset_run don't do anything different for inplace.
 * (They are not truly in place.)
 */



/* Compute the xor of src_1 and src_2 and write the result to
 * dst (which has no container initially).
 * Result is true iff dst is a bitset  */
bool array_bitset_container_xor(const array_container_t *src_1,
                                const bitset_container_t *src_2, void **dst);

/* Compute the xor of src_1 and src_2 and write the result to
 * dst. It is allowed for src_2 to be dst.  This version does not
 * update the cardinality of dst (it is set to BITSET_UNKNOWN_CARDINALITY).
 */

void array_bitset_container_lazy_xor(const array_container_t *src_1,
                                     const bitset_container_t *src_2,
                                     bitset_container_t *dst);
/* Compute the xor of src_1 and src_2 and write the result to
 * dst (which has no container initially). Return value is
 * "dst is a bitset"
 */

bool bitset_bitset_container_xor(const bitset_container_t *src_1,
                                 const bitset_container_t *src_2, void **dst);

/* Compute the xor of src_1 and src_2 and write the result to
 * dst. Result may be either a bitset or an array container
 * (returns "result is bitset"). dst does not initially have
 * any container, but becomes either a bitset container (return
 * result true) or an array container.
 */

bool run_bitset_container_xor(const run_container_t *src_1,
                              const bitset_container_t *src_2, void **dst);

/* lazy xor.  Dst is initialized and may be equal to src_2.
 *  Result is left as a bitset container, even if actual
 *  cardinality would dictate an array container.
 */

void run_bitset_container_lazy_xor(const run_container_t *src_1,
                                   const bitset_container_t *src_2,
                                   bitset_container_t *dst);

/* dst does not indicate a valid container initially.  Eventually it
 * can become any kind of container.
 */

int array_run_container_xor(const array_container_t *src_1,
                            const run_container_t *src_2, void **dst);

/* dst does not initially have a valid container.  Creates either
 * an array or a bitset container, indicated by return code
 */

bool array_array_container_xor(const array_container_t *src_1,
                               const array_container_t *src_2, void **dst);

/* dst does not initially have a valid container.  Creates either
 * an array or a bitset container, indicated by return code.
 * A bitset container will not have a valid cardinality and the
 * container type might not be correct for the actual cardinality
 */

bool array_array_container_lazy_xor(const array_container_t *src_1,
                                    const array_container_t *src_2, void **dst);

/* Dst is a valid run container. (Can it be src_2? Let's say not.)
 * Leaves result as run container, even if other options are
 * smaller.
 */

void array_run_container_lazy_xor(const array_container_t *src_1,
                                  const run_container_t *src_2,
                                  run_container_t *dst);

/* dst does not indicate a valid container initially.  Eventually it
 * can become any kind of container.
 */

int run_run_container_xor(const run_container_t *src_1,
                          const run_container_t *src_2, void **dst);

/* INPLACE versions (initial implementation may not exploit all inplace
 * opportunities (if any...)
 */

/* Compute the xor of src_1 and src_2 and write the result to
 * dst (which has no container initially).  It will modify src_1
 * to be dst if the result is a bitset.  Otherwise, it will
 * free src_1 and dst will be a new array container.  In both
 * cases, the caller is responsible for deallocating dst.
 * Returns true iff dst is a bitset  */

bool bitset_array_container_ixor(bitset_container_t *src_1,
                                 const array_container_t *src_2, void **dst);

bool bitset_bitset_container_ixor(bitset_container_t *src_1,
                                  const bitset_container_t *src_2, void **dst);

bool array_bitset_container_ixor(array_container_t *src_1,
                                 const bitset_container_t *src_2, void **dst);

/* Compute the xor of src_1 and src_2 and write the result to
 * dst. Result may be either a bitset or an array container
 * (returns "result is bitset"). dst does not initially have
 * any container, but becomes either a bitset container (return
 * result true) or an array container.
 */

bool run_bitset_container_ixor(run_container_t *src_1,
                               const bitset_container_t *src_2, void **dst);

bool bitset_run_container_ixor(bitset_container_t *src_1,
                               const run_container_t *src_2, void **dst);

/* dst does not indicate a valid container initially.  Eventually it
 * can become any kind of container.
 */

int array_run_container_ixor(array_container_t *src_1,
                             const run_container_t *src_2, void **dst);

int run_array_container_ixor(run_container_t *src_1,
                             const array_container_t *src_2, void **dst);

bool array_array_container_ixor(array_container_t *src_1,
                                const array_container_t *src_2, void **dst);

int run_run_container_ixor(run_container_t *src_1, const run_container_t *src_2,
                           void **dst);
#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/mixed_xor.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/containers.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/containers/containers.h"
#ifndef CONTAINERS_CONTAINERS_H
#define CONTAINERS_CONTAINERS_H

#include <assert.h>
#include <stdbool.h>
#include <stdio.h>


// would enum be possible or better?

/**
 * The switch case statements follow
 * BITSET_CONTAINER_TYPE_CODE -- ARRAY_CONTAINER_TYPE_CODE --
 * RUN_CONTAINER_TYPE_CODE
 * so it makes more sense to number them 1, 2, 3 (in the vague hope that the
 * compiler might exploit this ordering).
 */

#define BITSET_CONTAINER_TYPE_CODE 1
#define ARRAY_CONTAINER_TYPE_CODE 2
#define RUN_CONTAINER_TYPE_CODE 3
#define SHARED_CONTAINER_TYPE_CODE 4

// macro for pairing container type codes
#define CONTAINER_PAIR(c1, c2) (4 * (c1) + (c2))

/**
 * A shared container is a wrapper around a container
 * with reference counting.
 */

struct shared_container_s {
    void *container;
    uint8_t typecode;
    uint32_t counter;  // to be managed atomically
};

typedef struct shared_container_s shared_container_t;

/*
 * With copy_on_write = true
 *  Create a new shared container if the typecode is not SHARED_CONTAINER_TYPE,
 * otherwise, increase the count
 * If copy_on_write = false, then clone.
 * Return NULL in case of failure.
 **/
void *get_copy_of_container(void *container, uint8_t *typecode,
                            bool copy_on_write);

/* Frees a shared container (actually decrement its counter and only frees when
 * the counter falls to zero). */
void shared_container_free(shared_container_t *container);

/* extract a copy from the shared container, freeing the shared container if
there is just one instance left,
clone instances when the counter is higher than one
*/
void *shared_container_extract_copy(shared_container_t *container,
                                    uint8_t *typecode);

/* access to container underneath */
inline const void *container_unwrap_shared(
    const void *candidate_shared_container, uint8_t *type) {
    if (*type == SHARED_CONTAINER_TYPE_CODE) {
        *type =
            ((const shared_container_t *)candidate_shared_container)->typecode;
        assert(*type != SHARED_CONTAINER_TYPE_CODE);
        return ((shared_container_t *)candidate_shared_container)->container;
    } else {
        return candidate_shared_container;
    }
}

/* access to container underneath and queries its type */
static inline uint8_t get_container_type(const void *container, uint8_t type) {
    if (type == SHARED_CONTAINER_TYPE_CODE) {
        return ((shared_container_t *)container)->typecode;
    } else {
        return type;
    }
}

/**
 * Copies a container, requires a typecode. This allocates new memory, caller
 * is responsible for deallocation. If the container is not shared, then it is
 * physically cloned. Sharable containers are not cloneable.
 */
void *container_clone(const void *container, uint8_t typecode);

/* access to container underneath, cloning it if needed */
static inline void *get_writable_copy_if_shared(
    void *candidate_shared_container, uint8_t *type) {
    if (*type == SHARED_CONTAINER_TYPE_CODE) {
        return shared_container_extract_copy(
            (shared_container_t *)candidate_shared_container, type);
    } else {
        return candidate_shared_container;
    }
}

/**
 * End of shared container code
 */

static const char *container_names[] = {"bitset", "array", "run", "shared"};
static const char *shared_container_names[] = {
    "bitset (shared)", "array (shared)", "run (shared)"};

// no matter what the initial container was, convert it to a bitset
// if a new container is produced, caller responsible for freeing the previous
// one
// container should not be a shared container
static inline void *container_to_bitset(void *container, uint8_t typecode) {
    bitset_container_t *result = NULL;
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return container;  // nothing to do
        case ARRAY_CONTAINER_TYPE_CODE:
            result =
                bitset_container_from_array((array_container_t *)container);
            return result;
        case RUN_CONTAINER_TYPE_CODE:
            result = bitset_container_from_run((run_container_t *)container);
            return result;
        case SHARED_CONTAINER_TYPE_CODE:
            assert(false);
    }
    assert(false);
    __builtin_unreachable();
    return 0;  // unreached
}

/**
 * Get the container name from the typecode
 */
static inline const char *get_container_name(uint8_t typecode) {
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return container_names[0];
        case ARRAY_CONTAINER_TYPE_CODE:
            return container_names[1];
        case RUN_CONTAINER_TYPE_CODE:
            return container_names[2];
        case SHARED_CONTAINER_TYPE_CODE:
            return container_names[3];
        default:
            assert(false);
            __builtin_unreachable();
            return "unknown";
    }
}

static inline const char *get_full_container_name(void *container,
                                                  uint8_t typecode) {
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return container_names[0];
        case ARRAY_CONTAINER_TYPE_CODE:
            return container_names[1];
        case RUN_CONTAINER_TYPE_CODE:
            return container_names[2];
        case SHARED_CONTAINER_TYPE_CODE:
            switch (((shared_container_t *)container)->typecode) {
                case BITSET_CONTAINER_TYPE_CODE:
                    return shared_container_names[0];
                case ARRAY_CONTAINER_TYPE_CODE:
                    return shared_container_names[1];
                case RUN_CONTAINER_TYPE_CODE:
                    return shared_container_names[2];
                default:
                    assert(false);
                    __builtin_unreachable();
                    return "unknown";
            }
            break;
        default:
            assert(false);
            __builtin_unreachable();
            return "unknown";
    }
    __builtin_unreachable();
    return NULL;
}

/**
 * Get the container cardinality (number of elements), requires a  typecode
 */
static inline int container_get_cardinality(const void *container,
                                            uint8_t typecode) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_cardinality(
                (const bitset_container_t *)container);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_cardinality(
                (const array_container_t *)container);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_cardinality(
                (const run_container_t *)container);
    }
    assert(false);
    __builtin_unreachable();
    return 0;  // unreached
}

/*  Create a container with all the values between in [min,max) at a
    distance k*step from min. */
static inline void *container_from_range(uint8_t *type, uint32_t min,
                                         uint32_t max, uint16_t step) {
    if (step == 0) return NULL;  // being paranoid
    if (step == 1) {
        *type = RUN_CONTAINER_TYPE_CODE;
        return run_container_create_range(min, max);
    }
    int size = (max - min + step - 1) / step;
    if (size <= DEFAULT_MAX_SIZE) {  // array container
        *type = ARRAY_CONTAINER_TYPE_CODE;
        array_container_t *array = array_container_create_given_capacity(size);
        array_container_add_from_range(array, min, max, step);
        assert(array->cardinality == size);
        return array;
    } else {  // bitset container
        *type = BITSET_CONTAINER_TYPE_CODE;
        bitset_container_t *bitset = bitset_container_create();
        bitset_container_add_from_range(bitset, min, max, step);
        assert(bitset->cardinality == size);
        return bitset;
    }
}

/**
 * "repair" the container after lazy operations.
 */
static inline void *container_repair_after_lazy(void *container,
                                                uint8_t *typecode) {
    container = get_writable_copy_if_shared(
        container, typecode);  // TODO: this introduces unnecessary cloning
    void *result = NULL;
    switch (*typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            ((bitset_container_t *)container)->cardinality =
                bitset_container_compute_cardinality(
                    (bitset_container_t *)container);
            if (((bitset_container_t *)container)->cardinality <=
                DEFAULT_MAX_SIZE) {
                result = array_container_from_bitset(
                    (const bitset_container_t *)container);
                bitset_container_free((bitset_container_t *)container);
                *typecode = ARRAY_CONTAINER_TYPE_CODE;
                return result;
            }
            return container;
        case ARRAY_CONTAINER_TYPE_CODE:
            return container;  // nothing to do
        case RUN_CONTAINER_TYPE_CODE:
            return convert_run_to_efficient_container_and_free(
                (run_container_t *)container, typecode);
        case SHARED_CONTAINER_TYPE_CODE:
            assert(false);
    }
    assert(false);
    __builtin_unreachable();
    return 0;  // unreached
}

/**
 * Writes the underlying array to buf, outputs how many bytes were written.
 * This is meant to be byte-by-byte compatible with the Java and Go versions of
 * Roaring.
 * The number of bytes written should be
 * container_write(container, buf).
 *
 */
static inline int32_t container_write(const void *container, uint8_t typecode,
                                      char *buf) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_write((bitset_container_t *)container, buf);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_write((array_container_t *)container, buf);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_write((run_container_t *)container, buf);
    }
    assert(false);
    __builtin_unreachable();
    return 0;  // unreached
}

/**
 * Get the container size in bytes under portable serialization (see
 * container_write), requires a
 * typecode
 */
static inline int32_t container_size_in_bytes(const void *container,
                                              uint8_t typecode) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_size_in_bytes(
                (bitset_container_t *)container);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_size_in_bytes(
                (array_container_t *)container);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_size_in_bytes((run_container_t *)container);
    }
    assert(false);
    __builtin_unreachable();
    return 0;  // unreached
}

/**
 * print the container (useful for debugging), requires a  typecode
 */
void container_printf(const void *container, uint8_t typecode);

/**
 * print the content of the container as a comma-separated list of 32-bit values
 * starting at base, requires a  typecode
 */
void container_printf_as_uint32_array(const void *container, uint8_t typecode,
                                      uint32_t base);

/**
 * Checks whether a container is not empty, requires a  typecode
 */
static inline bool container_nonzero_cardinality(const void *container,
                                                 uint8_t typecode) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_nonzero_cardinality(
                (bitset_container_t *)container);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_nonzero_cardinality(
                (array_container_t *)container);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_nonzero_cardinality(
                (run_container_t *)container);
    }
    assert(false);
    __builtin_unreachable();
    return 0;  // unreached
}

/**
 * Recover memory from a container, requires a  typecode
 */
void container_free(void *container, uint8_t typecode);

/**
 * Convert a container to an array of values, requires a  typecode as well as a
 * "base" (most significant values)
 * Returns number of ints added.
 */
static inline int container_to_uint32_array(uint32_t *output,
                                            const void *container,
                                            uint8_t typecode, uint32_t base) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_to_uint32_array(
                output, (bitset_container_t *)container, base);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_to_uint32_array(
                output, (array_container_t *)container, base);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_to_uint32_array(
                output, (run_container_t *)container, base);
    }
    assert(false);
    __builtin_unreachable();
    return 0;  // unreached
}

/**
 * Add a value to a container, requires a  typecode, fills in new_typecode and
 * return (possibly different) container.
 * This function may allocate a new container, and caller is responsible for
 * memory deallocation
 */
static inline void *container_add(void *container, uint16_t val,
                                  uint8_t typecode, uint8_t *new_typecode) {
    container = get_writable_copy_if_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            bitset_container_set((bitset_container_t *)container, val);
            *new_typecode = BITSET_CONTAINER_TYPE_CODE;
            return container;
        case ARRAY_CONTAINER_TYPE_CODE: {
            array_container_t *ac = (array_container_t *)container;
            array_container_add(ac, val);
            if (array_container_cardinality(ac) > DEFAULT_MAX_SIZE) {
                *new_typecode = BITSET_CONTAINER_TYPE_CODE;
                return bitset_container_from_array(ac);
            } else {
                *new_typecode = ARRAY_CONTAINER_TYPE_CODE;
                return ac;
            }
        } break;
        case RUN_CONTAINER_TYPE_CODE:
            // per Java, no container type adjustments are done (revisit?)
            run_container_add((run_container_t *)container, val);
            *new_typecode = RUN_CONTAINER_TYPE_CODE;
            return container;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Remove a value from a container, requires a  typecode, fills in new_typecode
 * and
 * return (possibly different) container.
 * This function may allocate a new container, and caller is responsible for
 * memory deallocation
 */
static inline void *container_remove(void *container, uint16_t val,
                                     uint8_t typecode, uint8_t *new_typecode) {
    container = get_writable_copy_if_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            if (bitset_container_remove((bitset_container_t *)container, val)) {
                if (bitset_container_cardinality(
                        (bitset_container_t *)container) <= DEFAULT_MAX_SIZE) {
                    *new_typecode = ARRAY_CONTAINER_TYPE_CODE;
                    return array_container_from_bitset(
                        (bitset_container_t *)container);
                }
            }
            *new_typecode = typecode;
            return container;
        case ARRAY_CONTAINER_TYPE_CODE:
            *new_typecode = typecode;
            array_container_remove((array_container_t *)container, val);
            return container;
        case RUN_CONTAINER_TYPE_CODE:
            // per Java, no container type adjustments are done (revisit?)
            run_container_remove((run_container_t *)container, val);
            *new_typecode = RUN_CONTAINER_TYPE_CODE;
            return container;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Check whether a value is in a container, requires a  typecode
 */
inline bool container_contains(const void *container, uint16_t val,
                                      uint8_t typecode) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_get((const bitset_container_t *)container,
                                        val);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_contains(
                (const array_container_t *)container, val);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_contains((const run_container_t *)container,
                                          val);
        default:
            assert(false);
            __builtin_unreachable();
            return false;
    }
}

int32_t container_serialize(const void *container, uint8_t typecode,
                            char *buf) WARN_UNUSED;

uint32_t container_serialization_len(const void *container, uint8_t typecode);

void *container_deserialize(uint8_t typecode, const char *buf, size_t buf_len);

/**
 * Returns true if the two containers have the same content. Note that
 * two containers having different types can be "equal" in this sense.
 */
static inline bool container_equals(const void *c1, uint8_t type1,
                                    const void *c2, uint8_t type2) {
    c1 = container_unwrap_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            return bitset_container_equals((bitset_container_t *)c1,
                                           (bitset_container_t *)c2);
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            return run_container_equals_bitset((run_container_t *)c2,
                                               (bitset_container_t *)c1);
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            return run_container_equals_bitset((run_container_t *)c1,
                                               (bitset_container_t *)c2);
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            // java would always return false?
            return array_container_equal_bitset((array_container_t *)c2,
                                                (bitset_container_t *)c1);
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            // java would always return false?
            return array_container_equal_bitset((array_container_t *)c1,
                                                (bitset_container_t *)c2);
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            return run_container_equals_array((run_container_t *)c2,
                                              (array_container_t *)c1);
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            return run_container_equals_array((run_container_t *)c1,
                                              (array_container_t *)c2);
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            return array_container_equals((array_container_t *)c1,
                                          (array_container_t *)c2);
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            return run_container_equals((run_container_t *)c1,
                                        (run_container_t *)c2);
        default:
            assert(false);
            __builtin_unreachable();
            return false;
    }
}

// macro-izations possibilities for generic non-inplace binary-op dispatch

/**
 * Compute intersection between two containers, generate a new container (having
 * type result_type), requires a typecode. This allocates new memory, caller
 * is responsible for deallocation.
 */
static inline void *container_and(const void *c1, uint8_t type1, const void *c2,
                                  uint8_t type2, uint8_t *result_type) {
    c1 = container_unwrap_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = bitset_bitset_container_intersection(
                               (const bitset_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            result = array_container_create();
            array_container_intersection((const array_container_t *)c1,
                                         (const array_container_t *)c2,
                                         (array_container_t *)result);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            run_container_intersection((const run_container_t *)c1,
                                       (const run_container_t *)c2,
                                       (run_container_t *)result);
            return convert_run_to_efficient_container_and_free(
                (run_container_t *)result, result_type);
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            result = array_container_create();
            array_bitset_container_intersection((const array_container_t *)c2,
                                                (const bitset_container_t *)c1,
                                                (array_container_t *)result);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = array_container_create();
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            array_bitset_container_intersection((const array_container_t *)c1,
                                                (const bitset_container_t *)c2,
                                                (array_container_t *)result);
            return result;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            *result_type = run_bitset_container_intersection(
                               (const run_container_t *)c2,
                               (const bitset_container_t *)c1, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = run_bitset_container_intersection(
                               (const run_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = array_container_create();
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            array_run_container_intersection((const array_container_t *)c1,
                                             (const run_container_t *)c2,
                                             (array_container_t *)result);
            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            result = array_container_create();
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            array_run_container_intersection((const array_container_t *)c2,
                                             (const run_container_t *)c1,
                                             (array_container_t *)result);
            return result;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Compute intersection between two containers, with result in the first
 container if possible. If the returned pointer is identical to c1,
 then the container has been modified. If the returned pointer is different
 from c1, then a new container has been created and the caller is responsible
 for freeing it.
 The type of the first container may change. Returns the modified
 (and possibly new) container.
*/
static inline void *container_iand(void *c1, uint8_t type1, const void *c2,
                                   uint8_t type2, uint8_t *result_type) {
    c1 = get_writable_copy_if_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type =
                bitset_bitset_container_intersection_inplace(
                    (bitset_container_t *)c1, (bitset_container_t *)c2, &result)
                    ? BITSET_CONTAINER_TYPE_CODE
                    : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            array_container_intersection_inplace((array_container_t *)c1,
                                                 (const array_container_t *)c2);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;
            return c1;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            run_container_intersection((const run_container_t *)c1,
                                       (const run_container_t *)c2,
                                       (run_container_t *)result);
            // as of January 2016, Java code used non-in-place intersection for
            // two runcontainers
            return convert_run_to_efficient_container_and_free(
                (run_container_t *)result, result_type);
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            // c1 is a bitmap so no inplace possible
            result = array_container_create();
            array_bitset_container_intersection((const array_container_t *)c2,
                                                (const bitset_container_t *)c1,
                                                (array_container_t *)result);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            array_bitset_container_intersection(
                (const array_container_t *)c1, (const bitset_container_t *)c2,
                (array_container_t *)c1);  // allowed
            return c1;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            // will attempt in-place computation
            *result_type = run_bitset_container_intersection(
                               (const run_container_t *)c2,
                               (const bitset_container_t *)c1, &c1)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return c1;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = run_bitset_container_intersection(
                               (const run_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = array_container_create();
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            array_run_container_intersection((const array_container_t *)c1,
                                             (const run_container_t *)c2,
                                             (array_container_t *)result);
            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            result = array_container_create();
            *result_type = ARRAY_CONTAINER_TYPE_CODE;  // never bitset
            array_run_container_intersection((const array_container_t *)c2,
                                             (const run_container_t *)c1,
                                             (array_container_t *)result);
            return result;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Compute union between two containers, generate a new container (having type
 * result_type), requires a typecode. This allocates new memory, caller
 * is responsible for deallocation.
 */
static inline void *container_or(const void *c1, uint8_t type1, const void *c2,
                                 uint8_t type2, uint8_t *result_type) {
    c1 = container_unwrap_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            bitset_container_or((const bitset_container_t *)c1,
                                (const bitset_container_t *)c2,
                                (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = array_array_container_union(
                               (const array_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            run_container_union((const run_container_t *)c1,
                                (const run_container_t *)c2,
                                (run_container_t *)result);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // todo: could be optimized since will never convert to array
            result = convert_run_to_efficient_container_and_free(
                (run_container_t *)result, (uint8_t *)result_type);
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            array_bitset_container_union((const array_container_t *)c2,
                                         (const bitset_container_t *)c1,
                                         (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            array_bitset_container_union((const array_container_t *)c1,
                                         (const bitset_container_t *)c2,
                                         (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            if (run_container_is_full((run_container_t *)c2)) {
                result = run_container_create();
                *result_type = RUN_CONTAINER_TYPE_CODE;
                run_container_copy((const run_container_t *)c2,
                                   (run_container_t *)result);
                return result;
            }
            result = bitset_container_create();
            run_bitset_container_union((const run_container_t *)c2,
                                       (const bitset_container_t *)c1,
                                       (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c1)) {
                result = run_container_create();
                *result_type = RUN_CONTAINER_TYPE_CODE;
                run_container_copy((const run_container_t *)c1,
                                   (run_container_t *)result);
                return result;
            }
            result = bitset_container_create();
            run_bitset_container_union((const run_container_t *)c1,
                                       (const bitset_container_t *)c2,
                                       (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_union((const array_container_t *)c1,
                                      (const run_container_t *)c2,
                                      (run_container_t *)result);
            result = convert_run_to_efficient_container_and_free(
                (run_container_t *)result, (uint8_t *)result_type);
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_union((const array_container_t *)c2,
                                      (const run_container_t *)c1,
                                      (run_container_t *)result);
            result = convert_run_to_efficient_container_and_free(
                (run_container_t *)result, (uint8_t *)result_type);
            return result;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;  // unreached
    }
}

/**
 * Compute union between two containers, generate a new container (having type
 * result_type), requires a typecode. This allocates new memory, caller
 * is responsible for deallocation.
 *
 * This lazy version delays some operations such as the maintenance of the
 * cardinality. It requires repair later on the generated containers.
 */
static inline void *container_lazy_or(const void *c1, uint8_t type1,
                                      const void *c2, uint8_t type2,
                                      uint8_t *result_type) {
    c1 = container_unwrap_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            bitset_container_or_nocard(
                (const bitset_container_t *)c1, (const bitset_container_t *)c2,
                (bitset_container_t *)result);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = array_array_container_lazy_union(
                               (const array_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            run_container_union((const run_container_t *)c1,
                                (const run_container_t *)c2,
                                (run_container_t *)result);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // we are being lazy
            result = convert_run_to_efficient_container(
                (run_container_t *)result, result_type);
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            array_bitset_container_lazy_union(
                (const array_container_t *)c2, (const bitset_container_t *)c1,
                (bitset_container_t *)result);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            array_bitset_container_lazy_union(
                (const array_container_t *)c1, (const bitset_container_t *)c2,
                (bitset_container_t *)result);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c2)) {
                result = run_container_create();
                *result_type = RUN_CONTAINER_TYPE_CODE;
                run_container_copy((const run_container_t *)c2,
                                   (run_container_t *)result);
                return result;
            }
            result = bitset_container_create();
            run_bitset_container_lazy_union(
                (const run_container_t *)c2, (const bitset_container_t *)c1,
                (bitset_container_t *)result);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c1)) {
                result = run_container_create();
                *result_type = RUN_CONTAINER_TYPE_CODE;
                run_container_copy((const run_container_t *)c1,
                                   (run_container_t *)result);
                return result;
            }
            result = bitset_container_create();
            run_bitset_container_lazy_union(
                (const run_container_t *)c1, (const bitset_container_t *)c2,
                (bitset_container_t *)result);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_union((const array_container_t *)c1,
                                      (const run_container_t *)c2,
                                      (run_container_t *)result);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // next line skipped since we are lazy
            // result = convert_run_to_efficient_container(result, result_type);
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_union(
                (const array_container_t *)c2, (const run_container_t *)c1,
                (run_container_t *)result);  // TODO make lazy
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // next line skipped since we are lazy
            // result = convert_run_to_efficient_container(result, result_type);
            return result;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;  // unreached
    }
}

/**
 * Compute the union between two containers, with result in the first container.
 * If the returned pointer is identical to c1, then the container has been
 * modified.
 * If the returned pointer is different from c1, then a new container has been
 * created and the caller is responsible for freeing it.
 * The type of the first container may change. Returns the modified
 * (and possibly new) container
*/
static inline void *container_ior(void *c1, uint8_t type1, const void *c2,
                                  uint8_t type2, uint8_t *result_type) {
    c1 = get_writable_copy_if_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            bitset_container_or((const bitset_container_t *)c1,
                                (const bitset_container_t *)c2,
                                (bitset_container_t *)c1);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return c1;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            // Java impl. also does not do real in-place in this case
            *result_type = array_array_container_union(
                               (const array_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            run_container_union_inplace((run_container_t *)c1,
                                        (const run_container_t *)c2);
            return convert_run_to_efficient_container((run_container_t *)c1,
                                                      result_type);
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            array_bitset_container_union((const array_container_t *)c2,
                                         (const bitset_container_t *)c1,
                                         (bitset_container_t *)c1);
            *result_type = BITSET_CONTAINER_TYPE_CODE;  // never array
            return c1;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            // c1 is an array, so no in-place possible
            result = bitset_container_create();
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            array_bitset_container_union((const array_container_t *)c1,
                                         (const bitset_container_t *)c2,
                                         (bitset_container_t *)result);
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c2)) {
                result = run_container_create();
                *result_type = RUN_CONTAINER_TYPE_CODE;
                run_container_copy((const run_container_t *)c2,
                                   (run_container_t *)result);
                return result;
            }
            run_bitset_container_union((const run_container_t *)c2,
                                       (const bitset_container_t *)c1,
                                       (bitset_container_t *)c1);  // allowed
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return c1;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c1)) {
                *result_type = RUN_CONTAINER_TYPE_CODE;

                return c1;
            }
            result = bitset_container_create();
            run_bitset_container_union((const run_container_t *)c1,
                                       (const bitset_container_t *)c2,
                                       (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_union((const array_container_t *)c1,
                                      (const run_container_t *)c2,
                                      (run_container_t *)result);
            result = convert_run_to_efficient_container_and_free(
                (run_container_t *)result, result_type);
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            array_run_container_inplace_union((const array_container_t *)c2,
                                              (run_container_t *)c1);
            c1 = convert_run_to_efficient_container((run_container_t *)c1,
                                                    result_type);
            return c1;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Compute the union between two containers, with result in the first container.
 * If the returned pointer is identical to c1, then the container has been
 * modified.
 * If the returned pointer is different from c1, then a new container has been
 * created and the caller is responsible for freeing it.
 * The type of the first container may change. Returns the modified
 * (and possibly new) container
 *
 * This lazy version delays some operations such as the maintenance of the
 * cardinality. It requires repair later on the generated containers.
*/
static inline void *container_lazy_ior(void *c1, uint8_t type1, const void *c2,
                                       uint8_t type2, uint8_t *result_type) {
    assert(type1 != SHARED_CONTAINER_TYPE_CODE);
    // c1 = get_writable_copy_if_shared(c1,&type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            bitset_container_or_nocard((const bitset_container_t *)c1,
                                       (const bitset_container_t *)c2,
                                       (bitset_container_t *)c1);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return c1;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            // Java impl. also does not do real in-place in this case
            *result_type = array_array_container_lazy_union(
                               (const array_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            run_container_union_inplace((run_container_t *)c1,
                                        (const run_container_t *)c2);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            return convert_run_to_efficient_container((run_container_t *)c1,
                                                      result_type);
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            array_bitset_container_lazy_union(
                (const array_container_t *)c2, (const bitset_container_t *)c1,
                (bitset_container_t *)c1);              // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;  // never array
            return c1;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            // c1 is an array, so no in-place possible
            result = bitset_container_create();
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            array_bitset_container_lazy_union(
                (const array_container_t *)c1, (const bitset_container_t *)c2,
                (bitset_container_t *)result);  // is lazy
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c2)) {
                result = run_container_create();
                *result_type = RUN_CONTAINER_TYPE_CODE;
                run_container_copy((const run_container_t *)c2,
                                   (run_container_t *)result);
                return result;
            }
            run_bitset_container_lazy_union(
                (const run_container_t *)c2, (const bitset_container_t *)c1,
                (bitset_container_t *)c1);  // allowed //  lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return c1;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c1)) {
                *result_type = RUN_CONTAINER_TYPE_CODE;
                return c1;
            }
            result = bitset_container_create();
            run_bitset_container_lazy_union(
                (const run_container_t *)c1, (const bitset_container_t *)c2,
                (bitset_container_t *)result);  //  lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_union((const array_container_t *)c1,
                                      (const run_container_t *)c2,
                                      (run_container_t *)result);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // next line skipped since we are lazy
            // result = convert_run_to_efficient_container_and_free(result,
            // result_type);
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            array_run_container_inplace_union((const array_container_t *)c2,
                                              (run_container_t *)c1);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // next line skipped since we are lazy
            // result = convert_run_to_efficient_container_and_free(result,
            // result_type);
            return c1;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Compute symmetric difference (xor) between two containers, generate a new
 * container (having type result_type), requires a typecode. This allocates new
 * memory, caller is responsible for deallocation.
 */
static inline void *container_xor(const void *c1, uint8_t type1, const void *c2,
                                  uint8_t type2, uint8_t *result_type) {
    c1 = container_unwrap_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = bitset_bitset_container_xor(
                               (const bitset_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = array_array_container_xor(
                               (const array_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            *result_type =
                run_run_container_xor((const run_container_t *)c1,
                                      (const run_container_t *)c2, &result);
            return result;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = array_bitset_container_xor(
                               (const array_container_t *)c2,
                               (const bitset_container_t *)c1, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = array_bitset_container_xor(
                               (const array_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            *result_type = run_bitset_container_xor(
                               (const run_container_t *)c2,
                               (const bitset_container_t *)c1, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):

            *result_type = run_bitset_container_xor(
                               (const run_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;

        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            *result_type =
                array_run_container_xor((const array_container_t *)c1,
                                        (const run_container_t *)c2, &result);
            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            *result_type =
                array_run_container_xor((const array_container_t *)c2,
                                        (const run_container_t *)c1, &result);
            return result;

        default:
            assert(false);
            __builtin_unreachable();
            return NULL;  // unreached
    }
}

/**
 * Compute xor between two containers, generate a new container (having type
 * result_type), requires a typecode. This allocates new memory, caller
 * is responsible for deallocation.
 *
 * This lazy version delays some operations such as the maintenance of the
 * cardinality. It requires repair later on the generated containers.
 */
static inline void *container_lazy_xor(const void *c1, uint8_t type1,
                                       const void *c2, uint8_t type2,
                                       uint8_t *result_type) {
    c1 = container_unwrap_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            bitset_container_xor_nocard(
                (const bitset_container_t *)c1, (const bitset_container_t *)c2,
                (bitset_container_t *)result);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = array_array_container_lazy_xor(
                               (const array_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            // nothing special done yet.
            *result_type =
                run_run_container_xor((const run_container_t *)c1,
                                      (const run_container_t *)c2, &result);
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            array_bitset_container_lazy_xor((const array_container_t *)c2,
                                            (const bitset_container_t *)c1,
                                            (bitset_container_t *)result);
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            array_bitset_container_lazy_xor((const array_container_t *)c1,
                                            (const bitset_container_t *)c2,
                                            (bitset_container_t *)result);
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            run_bitset_container_lazy_xor((const run_container_t *)c2,
                                          (const bitset_container_t *)c1,
                                          (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = bitset_container_create();
            run_bitset_container_lazy_xor((const run_container_t *)c1,
                                          (const bitset_container_t *)c2,
                                          (bitset_container_t *)result);
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return result;

        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_lazy_xor((const array_container_t *)c1,
                                         (const run_container_t *)c2,
                                         (run_container_t *)result);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // next line skipped since we are lazy
            // result = convert_run_to_efficient_container(result, result_type);
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            result = run_container_create();
            array_run_container_lazy_xor((const array_container_t *)c2,
                                         (const run_container_t *)c1,
                                         (run_container_t *)result);
            *result_type = RUN_CONTAINER_TYPE_CODE;
            // next line skipped since we are lazy
            // result = convert_run_to_efficient_container(result, result_type);
            return result;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;  // unreached
    }
}

/**
 * Compute the xor between two containers, with result in the first container.
 * If the returned pointer is identical to c1, then the container has been
 * modified.
 * If the returned pointer is different from c1, then a new container has been
 * created and the caller is responsible for freeing it.
 * The type of the first container may change. Returns the modified
 * (and possibly new) container
*/
static inline void *container_ixor(void *c1, uint8_t type1, const void *c2,
                                   uint8_t type2, uint8_t *result_type) {
    c1 = get_writable_copy_if_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = bitset_bitset_container_ixor(
                               (bitset_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = array_array_container_ixor(
                               (array_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            *result_type = run_run_container_ixor(
                (run_container_t *)c1, (const run_container_t *)c2, &result);
            return result;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = bitset_array_container_ixor(
                               (bitset_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = array_bitset_container_ixor(
                               (array_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;

            return result;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            *result_type =
                bitset_run_container_ixor((bitset_container_t *)c1,
                                          (const run_container_t *)c2, &result)
                    ? BITSET_CONTAINER_TYPE_CODE
                    : ARRAY_CONTAINER_TYPE_CODE;

            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = run_bitset_container_ixor(
                               (run_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;

            return result;

        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            *result_type = array_run_container_ixor(
                (array_container_t *)c1, (const run_container_t *)c2, &result);
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            *result_type = run_array_container_ixor(
                (run_container_t *)c1, (const array_container_t *)c2, &result);
            return result;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Compute the xor between two containers, with result in the first container.
 * If the returned pointer is identical to c1, then the container has been
 * modified.
 * If the returned pointer is different from c1, then a new container has been
 * created and the caller is responsible for freeing it.
 * The type of the first container may change. Returns the modified
 * (and possibly new) container
 *
 * This lazy version delays some operations such as the maintenance of the
 * cardinality. It requires repair later on the generated containers.
*/
static inline void *container_lazy_ixor(void *c1, uint8_t type1, const void *c2,
                                        uint8_t type2, uint8_t *result_type) {
    assert(type1 != SHARED_CONTAINER_TYPE_CODE);
    // c1 = get_writable_copy_if_shared(c1,&type1);
    c2 = container_unwrap_shared(c2, &type2);
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            bitset_container_xor_nocard((bitset_container_t *)c1,
                                        (const bitset_container_t *)c2,
                                        (bitset_container_t *)c1);  // is lazy
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            return c1;
        // TODO: other cases being lazy, esp. when we know inplace not likely
        // could see the corresponding code for union
        default:
            return container_ixor(c1, type1, c2, type2, result_type);
    }
}

/**
 * Compute difference (andnot) between two containers, generate a new
 * container (having type result_type), requires a typecode. This allocates new
 * memory, caller is responsible for deallocation.
 */
static inline void *container_andnot(const void *c1, uint8_t type1,
                                     const void *c2, uint8_t type2,
                                     uint8_t *result_type) {
    c1 = container_unwrap_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = bitset_bitset_container_andnot(
                               (const bitset_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            result = array_container_create();
            array_array_container_andnot((const array_container_t *)c1,
                                         (const array_container_t *)c2,
                                         (array_container_t *)result);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c2)) {
                result = array_container_create();
                *result_type = ARRAY_CONTAINER_TYPE_CODE;
                return result;
            }
            *result_type =
                run_run_container_andnot((const run_container_t *)c1,
                                         (const run_container_t *)c2, &result);
            return result;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = bitset_array_container_andnot(
                               (const bitset_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            result = array_container_create();
            array_bitset_container_andnot((const array_container_t *)c1,
                                          (const bitset_container_t *)c2,
                                          (array_container_t *)result);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c2)) {
                result = array_container_create();
                *result_type = ARRAY_CONTAINER_TYPE_CODE;
                return result;
            }
            *result_type = bitset_run_container_andnot(
                               (const bitset_container_t *)c1,
                               (const run_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):

            *result_type = run_bitset_container_andnot(
                               (const run_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;

        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            if (run_container_is_full((const run_container_t *)c2)) {
                result = array_container_create();
                *result_type = ARRAY_CONTAINER_TYPE_CODE;
                return result;
            }
            result = array_container_create();
            array_run_container_andnot((const array_container_t *)c1,
                                       (const run_container_t *)c2,
                                       (array_container_t *)result);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;
            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            *result_type = run_array_container_andnot(
                (const run_container_t *)c1, (const array_container_t *)c2,
                &result);
            return result;

        default:
            assert(false);
            __builtin_unreachable();
            return NULL;  // unreached
    }
}

/**
 * Compute the andnot between two containers, with result in the first
 * container.
 * If the returned pointer is identical to c1, then the container has been
 * modified.
 * If the returned pointer is different from c1, then a new container has been
 * created and the caller is responsible for freeing it.
 * The type of the first container may change. Returns the modified
 * (and possibly new) container
*/
static inline void *container_iandnot(void *c1, uint8_t type1, const void *c2,
                                      uint8_t type2, uint8_t *result_type) {
    c1 = get_writable_copy_if_shared(c1, &type1);
    c2 = container_unwrap_shared(c2, &type2);
    void *result = NULL;
    switch (CONTAINER_PAIR(type1, type2)) {
        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = bitset_bitset_container_iandnot(
                               (bitset_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            array_array_container_iandnot((array_container_t *)c1,
                                          (const array_container_t *)c2);
            *result_type = ARRAY_CONTAINER_TYPE_CODE;
            return c1;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            *result_type = run_run_container_iandnot(
                (run_container_t *)c1, (const run_container_t *)c2, &result);
            return result;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            ARRAY_CONTAINER_TYPE_CODE):
            *result_type = bitset_array_container_iandnot(
                               (bitset_container_t *)c1,
                               (const array_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = ARRAY_CONTAINER_TYPE_CODE;

            array_bitset_container_iandnot((array_container_t *)c1,
                                           (const bitset_container_t *)c2);
            return c1;

        case CONTAINER_PAIR(BITSET_CONTAINER_TYPE_CODE,
                            RUN_CONTAINER_TYPE_CODE):
            *result_type = bitset_run_container_iandnot(
                               (bitset_container_t *)c1,
                               (const run_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;

            return result;

        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE,
                            BITSET_CONTAINER_TYPE_CODE):
            *result_type = run_bitset_container_iandnot(
                               (run_container_t *)c1,
                               (const bitset_container_t *)c2, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;

            return result;

        case CONTAINER_PAIR(ARRAY_CONTAINER_TYPE_CODE, RUN_CONTAINER_TYPE_CODE):
            *result_type = ARRAY_CONTAINER_TYPE_CODE;
            array_run_container_iandnot((array_container_t *)c1,
                                        (const run_container_t *)c2);
            return c1;
        case CONTAINER_PAIR(RUN_CONTAINER_TYPE_CODE, ARRAY_CONTAINER_TYPE_CODE):
            *result_type = run_array_container_iandnot(
                (run_container_t *)c1, (const array_container_t *)c2, &result);
            return result;
        default:
            assert(false);
            __builtin_unreachable();
            return NULL;
    }
}

/**
 * Visit all values x of the container once, passing (base+x,ptr)
 * to iterator. You need to specify a container and its type.
 * Returns true if the iteration should continue.
 */
static inline bool container_iterate(const void *container, uint8_t typecode,
                                     uint32_t base, roaring_iterator iterator,
                                     void *ptr) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_iterate(
                (const bitset_container_t *)container, base, iterator, ptr);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_iterate((const array_container_t *)container,
                                           base, iterator, ptr);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_iterate((const run_container_t *)container,
                                         base, iterator, ptr);
        default:
            assert(false);
            __builtin_unreachable();
    }
    assert(false);
    __builtin_unreachable();
    return false;
}

static inline void *container_not(const void *c, uint8_t typ,
                                  uint8_t *result_type) {
    c = container_unwrap_shared(c, &typ);
    void *result = NULL;
    switch (typ) {
        case BITSET_CONTAINER_TYPE_CODE:
            *result_type = bitset_container_negation(
                               (const bitset_container_t *)c, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case ARRAY_CONTAINER_TYPE_CODE:
            result = bitset_container_create();
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            array_container_negation((const array_container_t *)c,
                                     (bitset_container_t *)result);
            return result;
        case RUN_CONTAINER_TYPE_CODE:
            *result_type =
                run_container_negation((const run_container_t *)c, &result);
            return result;

        default:
            assert(false);
            __builtin_unreachable();
    }
    assert(false);
    __builtin_unreachable();
    return NULL;
}

static inline void *container_not_range(const void *c, uint8_t typ,
                                        uint32_t range_start,
                                        uint32_t range_end,
                                        uint8_t *result_type) {
    c = container_unwrap_shared(c, &typ);
    void *result = NULL;
    switch (typ) {
        case BITSET_CONTAINER_TYPE_CODE:
            *result_type =
                bitset_container_negation_range((const bitset_container_t *)c,
                                                range_start, range_end, &result)
                    ? BITSET_CONTAINER_TYPE_CODE
                    : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case ARRAY_CONTAINER_TYPE_CODE:
            *result_type =
                array_container_negation_range((const array_container_t *)c,
                                               range_start, range_end, &result)
                    ? BITSET_CONTAINER_TYPE_CODE
                    : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case RUN_CONTAINER_TYPE_CODE:
            *result_type = run_container_negation_range(
                (const run_container_t *)c, range_start, range_end, &result);
            return result;

        default:
            assert(false);
            __builtin_unreachable();
    }
    assert(false);
    __builtin_unreachable();
    return NULL;
}

static inline void *container_inot(void *c, uint8_t typ, uint8_t *result_type) {
    c = get_writable_copy_if_shared(c, &typ);
    void *result = NULL;
    switch (typ) {
        case BITSET_CONTAINER_TYPE_CODE:
            *result_type = bitset_container_negation_inplace(
                               (bitset_container_t *)c, &result)
                               ? BITSET_CONTAINER_TYPE_CODE
                               : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case ARRAY_CONTAINER_TYPE_CODE:
            // will never be inplace
            result = bitset_container_create();
            *result_type = BITSET_CONTAINER_TYPE_CODE;
            array_container_negation((array_container_t *)c,
                                     (bitset_container_t *)result);
            array_container_free((array_container_t *)c);
            return result;
        case RUN_CONTAINER_TYPE_CODE:
            *result_type =
                run_container_negation_inplace((run_container_t *)c, &result);
            return result;

        default:
            assert(false);
            __builtin_unreachable();
    }
    assert(false);
    __builtin_unreachable();
    return NULL;
}

static inline void *container_inot_range(void *c, uint8_t typ,
                                         uint32_t range_start,
                                         uint32_t range_end,
                                         uint8_t *result_type) {
    c = get_writable_copy_if_shared(c, &typ);
    void *result = NULL;
    switch (typ) {
        case BITSET_CONTAINER_TYPE_CODE:
            *result_type =
                bitset_container_negation_range_inplace(
                    (bitset_container_t *)c, range_start, range_end, &result)
                    ? BITSET_CONTAINER_TYPE_CODE
                    : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case ARRAY_CONTAINER_TYPE_CODE:
            *result_type =
                array_container_negation_range_inplace(
                    (array_container_t *)c, range_start, range_end, &result)
                    ? BITSET_CONTAINER_TYPE_CODE
                    : ARRAY_CONTAINER_TYPE_CODE;
            return result;
        case RUN_CONTAINER_TYPE_CODE:
            *result_type = run_container_negation_range_inplace(
                (run_container_t *)c, range_start, range_end, &result);
            return result;

        default:
            assert(false);
            __builtin_unreachable();
    }
    assert(false);
    __builtin_unreachable();
    return NULL;
}

/**
 * make a container with a run of ones
 */
/* initially always use a run container, even if an array might be
 * marginally
 * smaller */
static inline void *container_range_of_ones(uint32_t range_start,
                                            uint32_t range_end,
                                            uint8_t *result_type) {
    *result_type = RUN_CONTAINER_TYPE_CODE;
    return run_container_create_range(range_start, range_end);
}

/**
 * If the element of given rank is in this container, supposing that
 * the first
 * element has rank start_rank, then the function returns true and
 * sets element
 * accordingly.
 * Otherwise, it returns false and update start_rank.
 */
static inline bool container_select(const void *container, uint8_t typecode,
                                    uint32_t *start_rank, uint32_t rank,
                                    uint32_t *element) {
    container = container_unwrap_shared(container, &typecode);
    switch (typecode) {
        case BITSET_CONTAINER_TYPE_CODE:
            return bitset_container_select((bitset_container_t *)container,
                                           start_rank, rank, element);
        case ARRAY_CONTAINER_TYPE_CODE:
            return array_container_select((array_container_t *)container,
                                          start_rank, rank, element);
        case RUN_CONTAINER_TYPE_CODE:
            return run_container_select((run_container_t *)container,
                                        start_rank, rank, element);
        default:
            assert(false);
            __builtin_unreachable();
    }
    assert(false);
    __builtin_unreachable();
    return false;
}

#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/containers/containers.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/roaring_array.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/roaring_array.h"
#ifndef INCLUDE_ROARING_ARRAY_H
#define INCLUDE_ROARING_ARRAY_H

#include <assert.h>
#include <stdbool.h>
#include <stdint.h>

#define MAX_CONTAINERS 65536

#define SERIALIZATION_ARRAY_UINT32 1
#define SERIALIZATION_CONTAINER 2

enum {
    SERIAL_COOKIE_NO_RUNCONTAINER = 12346,
    SERIAL_COOKIE = 12347,
    NO_OFFSET_THRESHOLD = 4
};

/**
 * Roaring arrays are array-based key-value pairs having containers as values
 * and 16-bit integer keys. A roaring bitmap  might be implemented as such.
 */

// parallel arrays.  Element sizes quite different.
// Alternative is array
// of structs.  Which would have better
// cache performance through binary searches?

typedef struct roaring_array_s {
    int32_t size;
    int32_t allocation_size;
    void **containers;
    uint16_t *keys;
    uint8_t *typecodes;
} roaring_array_t;

/**
 * Create a new roaring array
 */
roaring_array_t *ra_create(void);

/**
 * Initialize an existing roaring array with the specified capacity (in number
 * of containers)
 */
bool ra_init_with_capacity(roaring_array_t *new_ra, uint32_t cap);

/**
 * Initialize with default capacity
 */
bool ra_init(roaring_array_t * t) ;

/**
 * Copies this roaring array, we assume that dest is not initialized
 */
bool ra_copy(const roaring_array_t *source, roaring_array_t * dest, bool copy_on_write);

/**
 * Copies this roaring array, we assume that dest is initialized
 */
bool ra_overwrite(const roaring_array_t *source, roaring_array_t * dest, bool copy_on_write);


/**
 * Frees the memory used by a roaring array
 */
void ra_clear(roaring_array_t *r);

/**
 * Frees the memory used by a roaring array, but does not free the containers
 */
void ra_clear_without_containers(roaring_array_t *r);


/**
 * Frees just the containers
 */
void ra_clear_containers(roaring_array_t *ra);

/**
 * Get the index corresponding to a 16-bit key
 */
inline int32_t ra_get_index(const roaring_array_t *ra, uint16_t x) {
    if ((ra->size == 0) || ra->keys[ra->size - 1] == x) return ra->size - 1;

    return binarySearch(ra->keys, (int32_t)ra->size, x);
}

/**
 * Retrieves the container at index i, filling in the typecode
 */
inline void *ra_get_container_at_index(const roaring_array_t *ra, uint16_t i,
                                              uint8_t *typecode) {
    *typecode = ra->typecodes[i];
    return ra->containers[i];
}

/**
 * Retrieves the key at index i
 */
uint16_t ra_get_key_at_index(const roaring_array_t *ra, uint16_t i);

/**
 * Add a new key-value pair at index i
 */
void ra_insert_new_key_value_at(roaring_array_t *ra, int32_t i, uint16_t key,
                                void *container, uint8_t typecode);

/**
 * Append a new key-value pair
 */
void ra_append(roaring_array_t *ra, uint16_t s, void *c, uint8_t typecode);

/**
 * Append a new key-value pair to ra, cloning (in COW sense) a value from sa
 * at index index
 */
void ra_append_copy(roaring_array_t *ra, const roaring_array_t *sa, uint16_t index,
                    bool copy_on_write);

/**
 * Append new key-value pairs to ra, cloning (in COW sense)  values from sa
 * at indexes
 * [start_index, uint16_t end_index)
 */
void ra_append_copy_range(roaring_array_t *ra, const roaring_array_t *sa,
                          uint16_t start_index, uint16_t end_index,
                          bool copy_on_write);

/** appends from sa to ra, ending with the greatest key that is
 * is less or equal stopping_key
 */
void ra_append_copies_until(roaring_array_t *ra, const roaring_array_t *sa,
                            uint16_t stopping_key, bool copy_on_write);

/** appends from sa to ra, starting with the smallest key that is
 * is strictly greater than before_start
 */

void ra_append_copies_after(roaring_array_t *ra, const roaring_array_t *sa,
                            uint16_t before_start, bool copy_on_write);

/**
 * Move the key-value pairs to ra from sa at indexes
 * [start_index, uint16_t end_index), old array should not be freed
 * (use ra_clear_without_containers)
 **/
void ra_append_move_range(roaring_array_t *ra, roaring_array_t *sa,
                          uint16_t start_index, uint16_t end_index);
/**
 * Append new key-value pairs to ra,  from sa at indexes
 * [start_index, uint16_t end_index)
 */
void ra_append_range(roaring_array_t *ra, roaring_array_t *sa,
                     uint16_t start_index, uint16_t end_index,
                     bool copy_on_write);

/**
 * Set the container at the corresponding index using the specified
 * typecode.
 */
inline void ra_set_container_at_index(const roaring_array_t *ra, int32_t i, void *c,
                               uint8_t typecode) {
    assert(i < ra->size);
    ra->containers[i] = c;
    ra->typecodes[i] = typecode;
}


/**
 * If needed, increase the capacity of the array so that it can fit k values
 * (at
 * least);
 */
bool extend_array(roaring_array_t *ra, int32_t k);

inline int32_t ra_get_size(const roaring_array_t *ra) { return ra->size; }

static inline int32_t ra_advance_until(const roaring_array_t *ra, uint16_t x,
                                       int32_t pos) {
    return advanceUntil(ra->keys, pos, ra->size, x);
}

int32_t ra_advance_until_freeing(roaring_array_t *ra, uint16_t x, int32_t pos);

void ra_downsize(roaring_array_t *ra, int32_t new_length);

inline void ra_replace_key_and_container_at_index(roaring_array_t *ra, int32_t i,
                                           uint16_t key, void *c,
                                           uint8_t typecode) {
    assert(i < ra->size);

    ra->keys[i] = key;
    ra->containers[i] = c;
    ra->typecodes[i] = typecode;
}

// write set bits to an array
void ra_to_uint32_array(const roaring_array_t *ra, uint32_t *ans);

/**
 * write a bitmap to a buffer. This is meant to be compatible with
 * the
 * Java and Go versions. Return the size in bytes of the serialized
 * output (which should be ra_portable_size_in_bytes(ra)).
 */
size_t ra_portable_serialize(const roaring_array_t *ra, char *buf);

/**
 * read a bitmap from a serialized version. This is meant to be compatible
 * with
 * the
 * Java and Go versions.
 */
bool ra_portable_deserialize(roaring_array_t * ra, const char *buf);

/**
 * How many bytes are required to serialize this bitmap (meant to be
 * compatible
 * with Java and Go versions)
 */
size_t ra_portable_size_in_bytes(const roaring_array_t *ra);

/**
 * return true if it contains at least one run container.
 */
bool ra_has_run_container(const roaring_array_t *ra);

/**
 * Size of the header when serializing (meant to be compatible
 * with Java and Go versions)
 */
uint32_t ra_portable_header_size(const roaring_array_t *ra);

/**
 * If the container at the index i is share, unshare it (creating a local
 * copy if needed).
 */
static inline void ra_unshare_container_at_index(roaring_array_t *ra, uint16_t i) {
    assert(i < ra->size);
    ra->containers[i] =
        get_writable_copy_if_shared(ra->containers[i], &ra->typecodes[i]);
}


/**
 * remove at index i, sliding over all entries after i
 */
void ra_remove_at_index(roaring_array_t *ra, int32_t i);

/**
 * remove at index i, sliding over all entries after i. Free removed container.
 */
void ra_remove_at_index_and_free(roaring_array_t *ra, int32_t i);

/**
 * remove a chunk of indices, sliding over entries after it
 */
// void ra_remove_index_range(roaring_array_t *ra, int32_t begin, int32_t end);

// used in inplace andNot only, to slide left the containers from
// the mutated RoaringBitmap that are after the largest container of
// the argument RoaringBitmap.  It is followed by a call to resize.
//
void ra_copy_range(roaring_array_t *ra, uint32_t begin, uint32_t end,
                   uint32_t new_begin);

#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/roaring_array.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/misc/configreport.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/misc/configreport.h"
/*
 * configreport.h
 *
 */

#ifndef INCLUDE_MISC_CONFIGREPORT_H_
#define INCLUDE_MISC_CONFIGREPORT_H_

#include <stddef.h>  // for size_t
#include <stdint.h>
#include <stdio.h>


#ifdef IS_X64
// useful for basic info (0)
static inline void native_cpuid(unsigned int *eax, unsigned int *ebx,
                                unsigned int *ecx, unsigned int *edx) {
    __asm volatile("cpuid"
                   : "=a"(*eax), "=b"(*ebx), "=c"(*ecx), "=d"(*edx)
                   : "0"(*eax), "2"(*ecx));
}

// CPUID instruction takes no parameters as CPUID implicitly uses the EAX
// register.
// The EAX register should be loaded with a value specifying what information to
// return
static inline void cpuinfo(int code, int *eax, int *ebx, int *ecx, int *edx) {
    __asm__ volatile("cpuid;"  //  call cpuid instruction
                     : "=a"(*eax), "=b"(*ebx), "=c"(*ecx),
                       "=d"(*edx)  // output equal to "movl  %%eax %1"
                     : "a"(code)   // input equal to "movl %1, %%eax"
                     //:"%eax","%ebx","%ecx","%edx"// clobbered register
                     );
}

static inline int computecacheline() {
    int eax = 0, ebx = 0, ecx = 0, edx = 0;
    cpuinfo((int)0x80000006, &eax, &ebx, &ecx, &edx);
    return ecx & 0xFF;
}

// this is quite imperfect, but can be handy
static inline const char *guessprocessor() {
    unsigned eax = 1, ebx = 0, ecx = 0, edx = 0;
    native_cpuid(&eax, &ebx, &ecx, &edx);
    const char *codename;
    switch (eax >> 4) {
        case 0x506E:
            codename = "Skylake";
            break;
        case 0x406C:
            codename = "CherryTrail";
            break;
        case 0x306D:
            codename = "Broadwell";
            break;
        case 0x306C:
            codename = "Haswell";
            break;
        case 0x306A:
            codename = "IvyBridge";
            break;
        case 0x206A:
        case 0x206D:
            codename = "SandyBridge";
            break;
        case 0x2065:
        case 0x206C:
        case 0x206F:
            codename = "Westmere";
            break;
        case 0x106E:
        case 0x106A:
        case 0x206E:
            codename = "Nehalem";
            break;
        case 0x1067:
        case 0x106D:
            codename = "Penryn";
            break;
        case 0x006F:
        case 0x1066:
            codename = "Merom";
            break;
        case 0x0066:
            codename = "Presler";
            break;
        case 0x0063:
        case 0x0064:
            codename = "Prescott";
            break;
        case 0x006D:
            codename = "Dothan";
            break;
        case 0x0366:
            codename = "Cedarview";
            break;
        case 0x0266:
            codename = "Lincroft";
            break;
        case 0x016C:
            codename = "Pineview";
            break;
        default:
            codename = "UNKNOWN";
            break;
    }
    return codename;
}

static inline void tellmeall() {
    printf("Intel processor:  %s\t", guessprocessor());

#ifdef __VERSION__
    printf(" compiler version: %s\t", __VERSION__);
#endif
    printf("\tBuild option USEAVX ");
#ifdef USEAVX
    printf("enabled\n");
#else
    printf("disabled\n");
#endif
#ifndef __AVX2__
    printf("AVX2 is NOT available.\n");
#endif

    if ((sizeof(int) != 4) || (sizeof(long) != 8)) {
        printf("number of bytes: int = %lu long = %lu \n",
               (long unsigned int)sizeof(size_t),
               (long unsigned int)sizeof(int));
    }
#if __LITTLE_ENDIAN__
// This is what we expect!
// printf("you have little endian machine");
#endif
#if __BIG_ENDIAN__
    printf("you have a big endian machine");
#endif
#if __CHAR_BIT__
    if (__CHAR_BIT__ != 8) printf("on your machine, chars don't have 8bits???");
#endif
    if (computecacheline() != 64)
        printf("cache line: %d bytes\n", computecacheline());
}
#else

static inline void tellmeall() {
    printf("Non-X64  processor\n");
#ifdef __arm__
    printf("ARM processor detected\n");
#endif
#ifdef __VERSION__
    printf(" compiler version: %s\t", __VERSION__);
#endif
    if ((sizeof(int) != 4) || (sizeof(long) != 8)) {
        printf("number of bytes: int = %lu long = %lu \n",
               (long unsigned int)sizeof(size_t),
               (long unsigned int)sizeof(int));
    }
#if __LITTLE_ENDIAN__
// This is what we expect!
// printf("you have little endian machine");
#endif
#if __BIG_ENDIAN__
    printf("you have a big endian machine");
#endif
#if __CHAR_BIT__
    if (__CHAR_BIT__ != 8) printf("on your machine, chars don't have 8bits???");
#endif
}

#endif

#endif /* INCLUDE_MISC_CONFIGREPORT_H_ */
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/misc/configreport.h */
/* begin file /home/dlemire/CVS/github/CRoaring/include/roaring/roaring.h */
#line 8 "/home/dlemire/CVS/github/CRoaring/include/roaring/roaring.h"
/*
An implementation of Roaring Bitmaps in C.
*/

#ifndef ROARING_H
#define ROARING_H
#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>

typedef struct roaring_bitmap_s {
    roaring_array_t high_low_container;
    bool copy_on_write; /* copy_on_write: whether you want to use copy-on-write
                         (saves memory and avoids
                         copies but needs more care in a threaded context). */
} roaring_bitmap_t;

/**
 * Creates a new bitmap (initially empty)
 */
roaring_bitmap_t *roaring_bitmap_create(void);

/**
 * Add all the values between min (included) and max (excluded) that are at a
 * distance k*step from min.
*/
roaring_bitmap_t *roaring_bitmap_from_range(uint32_t min, uint32_t max,
                                            uint32_t step);

/**
 * Creates a new bitmap (initially empty) with a provided
 * container-storage capacity (it is a performance hint).
 */
roaring_bitmap_t *roaring_bitmap_create_with_capacity(uint32_t cap);

/**
 * Creates a new bitmap from a pointer of uint32_t integers
 */
roaring_bitmap_t *roaring_bitmap_of_ptr(size_t n_args, const uint32_t *vals);

/**
 * Describe the inner structure of the bitmap.
 */
void roaring_bitmap_printf_describe(const roaring_bitmap_t *ra);

/**
 * Creates a new bitmap from a list of uint32_t integers
 */
roaring_bitmap_t *roaring_bitmap_of(size_t n, ...);

/**
 * Copies a  bitmap. This does memory allocation. The caller is responsible for
 * memory management.
 *
 */
roaring_bitmap_t *roaring_bitmap_copy(const roaring_bitmap_t *r);

/**
 * Print the content of the bitmap.
 */
void roaring_bitmap_printf(const roaring_bitmap_t *ra);

/**
 * Computes the intersection between two bitmaps and returns new bitmap. The
 * caller is
 * responsible for memory management.
 *
 */
roaring_bitmap_t *roaring_bitmap_and(const roaring_bitmap_t *x1,
                                     const roaring_bitmap_t *x2);

/**
 * Inplace version modifies x1, x1 == x2 is allowed
 */
void roaring_bitmap_and_inplace(roaring_bitmap_t *x1,
                                const roaring_bitmap_t *x2);

/**
 * Computes the union between two bitmaps and returns new bitmap. The caller is
 * responsible for memory management.
 */
roaring_bitmap_t *roaring_bitmap_or(const roaring_bitmap_t *x1,
                                    const roaring_bitmap_t *x2);

/**
 * Inplace version of roaring_bitmap_or, modifies x1. TDOO: decide whether x1 ==
 *x2 ok
 *
 */
void roaring_bitmap_or_inplace(roaring_bitmap_t *x1,
                               const roaring_bitmap_t *x2);

/**
 * Compute the union of 'number' bitmaps. See also roaring_bitmap_or_many_heap.
 * Caller is responsible for freeing the
 * result.
 *
 */
roaring_bitmap_t *roaring_bitmap_or_many(size_t number,
                                         const roaring_bitmap_t **x);

/**
 * Compute the union of 'number' bitmaps using a heap. This can
 * sometimes be faster than roaring_bitmap_or_many which uses
 * a naive algorithm. Caller is responsible for freeing the
 * result.
 *
 */
roaring_bitmap_t *roaring_bitmap_or_many_heap(uint32_t number,
                                              const roaring_bitmap_t **x);

/**
 * Computes the symmetric difference (xor) between two bitmaps
 * and returns new bitmap. The caller is responsible for memory management.
 */
roaring_bitmap_t *roaring_bitmap_xor(const roaring_bitmap_t *x1,
                                     const roaring_bitmap_t *x2);

/**
 * Inplace version of roaring_bitmap_xor, modifies x1. x1 != x2.
 *
 */
void roaring_bitmap_xor_inplace(roaring_bitmap_t *x1,
                                const roaring_bitmap_t *x2);

/**
 * Compute the xor of 'number' bitmaps. See also roaring_bitmap_xor_many_heap.
 * Caller is responsible for freeing the
 * result.
 *
 */
roaring_bitmap_t *roaring_bitmap_xor_many(size_t number,
                                          const roaring_bitmap_t **x);

/**
 * Computes the  difference (andnot) between two bitmaps
 * and returns new bitmap. The caller is responsible for memory management.
 */
roaring_bitmap_t *roaring_bitmap_andnot(const roaring_bitmap_t *x1,
                                        const roaring_bitmap_t *x2);

/**
 * Inplace version of roaring_bitmap_andnot, modifies x1. x1 != x2.
 *
 */
void roaring_bitmap_andnot_inplace(roaring_bitmap_t *x1,
                                   const roaring_bitmap_t *x2);

/**
 * Compute the xor of 'number' bitmaps using a heap. This can
 * sometimes be faster than roaring_bitmap_xor_many which uses
 * a naive algorithm. Caller is responsible for freeing the
 * result.
 *
 * TODO: consider implementing
 * roaring_bitmap_t *roaring_bitmap_xor_many_heap(uint32_t number,
 *                                              const roaring_bitmap_t **x);
 */

/**
 * Frees the memory.
 */
void roaring_bitmap_free(roaring_bitmap_t *r);

/**
 * Add value x
 *
 */
void roaring_bitmap_add(roaring_bitmap_t *r, uint32_t x);

/**
 * Remove value x
 *
 */
void roaring_bitmap_remove(roaring_bitmap_t *r, uint32_t x);

/**
 * Check if value x is present
 */
inline bool roaring_bitmap_contains(const roaring_bitmap_t *r,
                                           uint32_t val) {
    const uint16_t hb = val >> 16;
    /*
     * here it is possible to bypass the binary search and the ra_get_index
     * call with the following call that might often come true
     */
    int32_t i = ra_get_index(& r->high_low_container, hb);
    if (i < 0) return false;

    uint8_t typecode;
    // next call ought to be cheap
    void *container =
        ra_get_container_at_index(& r->high_low_container, i, &typecode);
    // rest might be a tad expensive
    return container_contains(container, val & 0xFFFF, typecode);
}

/**
 * Get the cardinality of the bitmap (number of elements).
 */
uint64_t roaring_bitmap_get_cardinality(const roaring_bitmap_t *ra);

/**
* Returns true if the bitmap is empty (cardinality is zero).
*/
bool roaring_bitmap_is_empty(const roaring_bitmap_t *ra);

/**
 * Convert the bitmap to an array. Write the output to "ans",
 * caller is responsible to ensure that there is enough memory
 * allocated
 * (e.g., ans = malloc(roaring_bitmap_get_cardinality(mybitmap)
 *   * sizeof(uint32_t))
 */
void roaring_bitmap_to_uint32_array(const roaring_bitmap_t *ra, uint32_t *ans);

/**
 *  Remove run-length encoding even when it is more space efficient
 *  return whether a change was applied
 */
bool roaring_bitmap_remove_run_compression(roaring_bitmap_t *r);

/** convert array and bitmap containers to run containers when it is more
 * efficient;
 * also convert from run containers when more space efficient.  Returns
 * true if the result has at least one run container.
*/
bool roaring_bitmap_run_optimize(roaring_bitmap_t *r);

//
// write the bitmap to an output pointer, this output buffer should refer to
// at least roaring_bitmap_size_in_bytes(ra) allocated bytes.
//
// see roaring_bitmap_portable_serialize if you want a format that's compatible
// with Java and Go implementations
//
// this format has the benefit of being sometimes more space efficient than roaring_bitmap_portable_serialize
// e.g., when the data is sparse.
//
// Returns how many bytes were written which should be
// roaring_bitmap_size_in_bytes(ra).
size_t roaring_bitmap_serialize(const roaring_bitmap_t *ra, char *buf);

//  use with roaring_bitmap_serialize
// see roaring_bitmap_portable_deserialize if you want a format that's
// compatible with Java and Go implementations
roaring_bitmap_t *roaring_bitmap_deserialize(const void *buf);


/**
 * How many bytes are required to serialize this bitmap (NOT compatible
 * with Java and Go versions)
 */
size_t roaring_bitmap_size_in_bytes(const roaring_bitmap_t *ra);


/**
 * read a bitmap from a serialized version. This is meant to be compatible with
 * the
 * Java and Go versions.
 */
roaring_bitmap_t *roaring_bitmap_portable_deserialize(const char *buf);


/**
 * How many bytes are required to serialize this bitmap (meant to be compatible
 * with Java and Go versions)
 */
size_t roaring_bitmap_portable_size_in_bytes(const roaring_bitmap_t *ra);

/**
 * write a bitmap to a char buffer.  The output buffer should refer to at least
 *  roaring_bitmap_portable_size_in_bytes(ra) bytes of allocated memory.
 * This is meant to be compatible with
 * the
 * Java and Go versions. Returns how many bytes were written which should be
 * roaring_bitmap_portable_size_in_bytes(ra).
 */
size_t roaring_bitmap_portable_serialize(const roaring_bitmap_t *ra, char *buf);

/**
 * Iterate over the bitmap elements. The function iterator is called once for
 *  all the values with ptr (can be NULL) as the second parameter of each call.
 *
 *  roaring_iterator is simply a pointer to a function that returns bool
 *  (true means that the iteration should continue while false means that it
 * should stop),
 *  and takes (uint32_t,void*) as inputs.
 *
 *  Returns true if the roaring_iterator returned true throughout (so that
 *  all data points were necessarily visited).
 */
bool roaring_iterate(const roaring_bitmap_t *ra, roaring_iterator iterator,
                     void *ptr);

/**
 * Return true if the two bitmaps contain the same elements.
 */
bool roaring_bitmap_equals(roaring_bitmap_t *ra1, roaring_bitmap_t *ra2);

/**
 * (For expert users who seek high performance.)
 *
 * Computes the union between two bitmaps and returns new bitmap. The caller is
 * responsible for memory management.
 *
 * The lazy version defers some computations such as the maintenance of the
 * cardinality counts. Thus you need
 * to call roaring_bitmap_repair_after_lazy after executing "lazy" computations.
 * It is safe to repeatedly call roaring_bitmap_lazy_or_inplace on the result.
 * The bitsetconversion conversion is a flag which determines
 * whether container-container operations force a bitset conversion.
 **/
roaring_bitmap_t *roaring_bitmap_lazy_or(const roaring_bitmap_t *x1,
                                         const roaring_bitmap_t *x2,
                                         const bool bitsetconversion);

/**
 * (For expert users who seek high performance.)
 * Inplace version of roaring_bitmap_lazy_or, modifies x1
 * The bitsetconversion conversion is a flag which determines
 * whether container-container operations force a bitset conversion.
 */
void roaring_bitmap_lazy_or_inplace(roaring_bitmap_t *x1,
                                    const roaring_bitmap_t *x2,
                                    const bool bitsetconversion);

/**
 * (For expert users who seek high performance.)
 *
 * Execute maintenance operations on a bitmap created from
 * roaring_bitmap_lazy_or
 * or modified with roaring_bitmap_lazy_or_inplace.
 */
void roaring_bitmap_repair_after_lazy(roaring_bitmap_t *x1);

/**
 * Computes the symmetric difference between two bitmaps and returns new bitmap.
 *The caller is
 * responsible for memory management.
 *
 * The lazy version defers some computations such as the maintenance of the
 * cardinality counts. Thus you need
 * to call roaring_bitmap_repair_after_lazy after executing "lazy" computations.
 * It is safe to repeatedly call roaring_bitmap_lazy_xor_inplace on the result.
 *
 */
roaring_bitmap_t *roaring_bitmap_lazy_xor(const roaring_bitmap_t *x1,
                                          const roaring_bitmap_t *x2);

/**
 * (For expert users who seek high performance.)
 * Inplace version of roaring_bitmap_lazy_xor, modifies x1. x1 != x2
 *
 */
void roaring_bitmap_lazy_xor_inplace(roaring_bitmap_t *x1,
                                     const roaring_bitmap_t *x2);

/**
 * compute the negation of the roaring bitmap within a specified interval.
 * areas outside the range are passed through unchanged.
 */

roaring_bitmap_t *roaring_bitmap_flip(const roaring_bitmap_t *x1,
                                      uint64_t range_start, uint64_t range_end);

/**
 * compute (in place) the negation of the roaring bitmap within a specified
 * interval.
 * areas outside the range are passed through unchanged.
 */

void roaring_bitmap_flip_inplace(roaring_bitmap_t *x1, uint64_t range_start,
                                 uint64_t range_end);

/**
 * If the size of the roaring bitmap is strictly greater than rank, then this
   function returns true and set element to the element of given rank.
   Otherwise, it returns false.
 */
bool roaring_bitmap_select(const roaring_bitmap_t *ra, uint32_t rank,
                           uint32_t *element);

/**
*  (For advanced users.)
* Collect statistics about the bitmap, see roaring_types.h for
* a description of roaring_statistics_t
*/
void roaring_bitmap_statistics(const roaring_bitmap_t *ra,
                               roaring_statistics_t *stat);

#ifdef __cplusplus
}
#endif

#endif
/* end file /home/dlemire/CVS/github/CRoaring/include/roaring/roaring.h */
