//
// Created by semyon on 04.06.19.
//

#ifndef CODEGEN_JUGR_H
#define CODEGEN_JUGR_H

#include <anymap.h>

namespace jugr {
    extern "C" int MAC_DLLEXPORT loadJUGRDataFromTSK(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT predicate_input(com::Anymap &data);

    extern "C" bool MAC_DLLEXPORT predicate_1(com::Anymap &data);

    extern "C" bool MAC_DLLEXPORT predicate_2(com::Anymap &data);

    extern "C" bool MAC_DLLEXPORT predicate_3(com::Anymap &data);

    extern "C" bool MAC_DLLEXPORT predicate_4(com::Anymap &data);


    extern "C" int MAC_DLLEXPORT function_input(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_1(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_2(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_3(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_12(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_13(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_24(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_34(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_41(com::Anymap &data);

    extern "C" int MAC_DLLEXPORT function_final(com::Anymap &data);


} // namespace jugr

#endif //CODEGEN_JUGR_H
