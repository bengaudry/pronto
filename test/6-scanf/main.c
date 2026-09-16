#include <stdio.h>

int main(void) {
    char name[100];
    int age;
    printf("Enter name: ");
    fflush(stdout);
    if (scanf("%99s", name) != 1) return 1;
    printf("Enter age: ");
    fflush(stdout);
    if (scanf("%d", &age) != 1) return 1;
    printf("Hello %s, you are %d\n", name, age);
    return 0;
}
