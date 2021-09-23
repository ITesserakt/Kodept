#ifndef JUGR_H_
#define JUGR_H_

#include <anymap.h>

namespace jugr {
extern "C" int MAC_DLLEXPORT loadJUGRDataFromTSK(com::Anymap &data);

extern "C" bool MAC_DLLEXPORT predicate_1(com::Anymap &data);

extern "C" bool MAC_DLLEXPORT predicate_2(com::Anymap &data);

extern "C" bool MAC_DLLEXPORT predicate_3(com::Anymap &data);

extern "C" int MAC_DLLEXPORT function_1(com::Anymap &data);

extern "C" int MAC_DLLEXPORT function_2(com::Anymap &data);

extern "C" int MAC_DLLEXPORT function_3(com::Anymap &data);

} // namespace jugr

#endif
