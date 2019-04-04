#include "test_R.h"

#include "stringtools.h"
#include <anymap.h>
#include <iniparser.h>

#include <mutex>
#include <thread>

using namespace std;
using namespace com;
using namespace ini;

int loadDataFromTSK(com::Anymap &data) {
  printf("loading data...\n");

  return 0;
}

bool test_R::predicate_1(com::Anymap &data) {
  printf("predicate_1\n");

  return false;
}

bool test_R::predicate_2(Anymap &data) {
  printf("predicate_2\n");

  return false;
}

bool test_R::predicate_3(Anymap &data) {
  printf("predicate_3\n");

  return true;
}

int test_R::function_2(Anymap &data) {
  printf("function_2\n");

  return 2;
}
int test_R::function_1(Anymap &data) {
  printf("function_1\n");

  return 1;
}

int test_R::function_3(Anymap &data) {
  printf("function_3\n");

  return 3;
}
