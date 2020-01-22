#ifndef TEST_R_H_
#define TEST_R_H_

#include <anymap.h>

namespace test_R {
extern "C" int MAC_DLLEXPORT loadDataFromTSK(com::Anymap &data);

extern "C" bool MAC_DLLEXPORT predicate_1(com::Anymap &data);

extern "C" bool MAC_DLLEXPORT predicate_2(com::Anymap &data);

extern "C" bool MAC_DLLEXPORT predicate_3(com::Anymap &data);

extern "C" int MAC_DLLEXPORT function_1(com::Anymap &data);

extern "C" int MAC_DLLEXPORT function_2(com::Anymap &data);

extern "C" int MAC_DLLEXPORT function_3(com::Anymap &data);

}

#endif
