#include <stdio.h>

typedef enum { FOOD, TOOL, TOY } Category;

static const char *category_label(Category c) {
    switch (c) {
    case FOOD:
        return "food";
    case TOOL:
        return "tool";
    case TOY:
        return "toy";
    }
    return "";
}

typedef struct {
    const char *name;
    Category category;
    long price;
    long qty;
} Item;

static long item_value(const Item *item) {
    return item->price * item->qty;
}

static long restock(Item *items, size_t count, long min) {
    long added = 0;
    for (size_t i = 0; i < count; i++) {
        if (items[i].qty < min) {
            added += min - items[i].qty;
            items[i].qty = min;
        }
    }
    return added;
}

int main(void) {
    Item items[] = {
        {.name = "apple", .category = FOOD, .price = 3, .qty = 10},
        {.name = "hammer", .category = TOOL, .price = 25, .qty = 1},
        {.name = "ball", .category = TOY, .price = 8, .qty = 0},
    };
    size_t count = sizeof items / sizeof items[0];
    long added = restock(items, count, 2);
    long total = 0;
    for (size_t i = 0; i < count; i++) {
        printf("%s (%s): %ld x %ld\n", items[i].name, category_label(items[i].category), items[i].qty, items[i].price);
        total += item_value(&items[i]);
    }
    printf("restocked %ld, stock value %ld\n", added, total);
    return 0;
}
