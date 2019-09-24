#include "stringtools.h"
#include <anymap.h>
#include <iniparser.h>

#include <mutex>
#include <thread>



int loadJUGRDataFromTSK(com::Anymap &data) {
    printf("loading data...\n");

    return 0;
}

bool predicate_1(com::Anymap &data) {
    printf("predicate_1\n");

    return false;
}

bool predicate_2(com::Anymap &data) {
    printf("predicate_2\n");

    return false;
}

bool predicate_3(com::Anymap &data) {
    printf("predicate_3\n");

    return true;
}

int function_2(com::Anymap &data) {
    printf("function_2\n");

    return 2;
}
int function_1(com::Anymap &data) {
    printf("function_1\n");

    return 1;
}

int function_3(com::Anymap &data) {
    printf("function_3\n");

    return 3;
}