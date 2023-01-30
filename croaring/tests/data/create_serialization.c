#include <assert.h>
#include <roaring.h>
#include <stdio.h>

void write_file(const char *path, const char *contents, size_t len) {
    FILE *f = fopen(path, "wb");
    assert(f != NULL);
    size_t n = fwrite(contents, 1, len, f);
    assert(n == len);
    fclose(f);
}

void write_frozen(const roaring_bitmap_t *b) {
    size_t size = roaring_bitmap_frozen_size_in_bytes(b);
    char *data = roaring_malloc(size);
    roaring_bitmap_frozen_serialize(b, data);
    write_file("frozen_bitmap.bin", data, size);
    roaring_free(data);
}

void write_portable(const roaring_bitmap_t *b) {
    size_t size = roaring_bitmap_portable_size_in_bytes(b);
    char *data = roaring_malloc(size);
    roaring_bitmap_portable_serialize(b, data);
    write_file("portable_bitmap.bin", data, size);
    roaring_free(data);
}

int main(void) {
    int i;

    roaring_bitmap_t *b = roaring_bitmap_create();
    // Range container
    roaring_bitmap_add_range(b, 0x00000, 0x09000);
    roaring_bitmap_add_range(b, 0x0A000, 0x10000);
    // Array container
    roaring_bitmap_add(b, 0x20000);
    roaring_bitmap_add(b, 0x20005);
    // Bitmap container
    for (i = 0; i < 0x10000; i += 2) {
      roaring_bitmap_add(b, 0x80000 + i);
    }

    roaring_bitmap_run_optimize(b);

    write_frozen(b);
    write_portable(b);

    roaring_bitmap_free(b);
}