#include <stdio.h>

typedef enum { GRADE_A, GRADE_B, GRADE_C, GRADE_F } Grade;

static const char *grade_label(Grade g) {
    switch (g) {
    case GRADE_A:
        return "excellent";
    case GRADE_B:
        return "good";
    case GRADE_C:
        return "passing";
    case GRADE_F:
        return "failing";
    }
    return "";
}

typedef struct {
    const char *name;
    long score;
} Student;

static Grade student_grade(const Student *s) {
    if (s->score >= 90) {
        return GRADE_A;
    } else if (s->score >= 80) {
        return GRADE_B;
    } else if (s->score >= 70) {
        return GRADE_C;
    } else {
        return GRADE_F;
    }
}

int main(void) {
    Student students[] = {
        {.name = "Ada", .score = 95},
        {.name = "Linus", .score = 82},
        {.name = "Grace", .score = 67},
    };
    size_t count = sizeof students / sizeof students[0];
    long total = 0;
    for (size_t i = 0; i < count; i++) {
        printf("%s: %ld (%s)\n", students[i].name, students[i].score, grade_label(student_grade(&students[i])));
        total += students[i].score;
    }
    printf("class average: %ld\n", total / (long)count);
    return 0;
}
