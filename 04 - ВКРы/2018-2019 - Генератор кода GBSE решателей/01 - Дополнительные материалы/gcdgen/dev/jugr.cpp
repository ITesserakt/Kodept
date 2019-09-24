#include "jugr.h"

#include "stringtools.h"
#include <anymap.h>
#include <iniparser.h>

#include <mutex>
#include <thread>

using namespace std;
using namespace com;
using namespace ini;

int loadJUGRDataFromTSK(com::Anymap &data) {
    printf("loading data...\n");

    return 0;
}

bool jugr::predicate_input(com::Anymap &data) {
    printf("predicate\n");

    return false;
}

bool jugr::function_input(com::Anymap &data) {
    printf("predicate\n");

    return false;
}

bool jugr::predicate_1(com::Anymap &data) {
    printf("predicate_1\n");

    return false;
}

bool jugr::predicate_2(Anymap &data) {
    printf("predicate_2\n");

    return false;
}

bool jugr::predicate_3(Anymap &data) {
    printf("predicate_3\n");

    return true;
}

bool jugr::predicate_4(com::Anymap &data) {
    printf("predicate\n");

    return false;
}

int jugr::function_2(Anymap &data) {
    printf("function_2\n");

    return 2;
}
int jugr::function_1(Anymap &data) {
    printf("function_1\n");

    return 1;
}

int jugr::function_3(Anymap &data) {
    printf("function_3\n");

    return 3;
}

int jugr::function_13(Anymap &data) {
    printf("function_3\n");

    return 3;
}

int jugr::function_12(Anymap &data) {
    printf("function_3\n");

    return 3;
}

int jugr::function_34(Anymap &data) {
    printf("function_3\n");

    return 3;
}

int jugr::function_24(Anymap &data) {
    printf("function_3\n");

    return 3;
}

int jugr::function_41(Anymap &data) {
    printf("function_3\n");

    return 3;
}

int jugr::function_final(Anymap &data) {
    printf("function_3\n");

    return 3;
}