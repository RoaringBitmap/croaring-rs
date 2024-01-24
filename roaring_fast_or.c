#include "croaring-sys/CRoaring/roaring.c"

int main(void) {
    roaring_bitmap_t *r1 = roaring_bitmap_of(2, 500, 1000);
    roaring_bitmap_t *r2 = roaring_bitmap_of(2, 1000, 2000);

    fprintf(stderr, "Hardware support: %#x\n", croaring_hardware_support());

    const roaring_bitmap_t *bitmap_arr[2] = {r1, r2};
    fprintf(stderr, "Going to or many\n");
    for (int i = 0; i < 10000; i++) {
        roaring_bitmap_t *r = roaring_bitmap_or_many(2, bitmap_arr);
        roaring_bitmap_free(r);
    }

    fprintf(stderr, "Got done\n");

    roaring_bitmap_free(r2);
    roaring_bitmap_free(r1);
}