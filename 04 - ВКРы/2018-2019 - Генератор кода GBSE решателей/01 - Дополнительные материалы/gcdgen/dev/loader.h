//
// Created by semyon on 20.05.19.
//

#ifndef CODEGEN_PROJ_LOADER_H
#define CODEGEN_PROJ_LOADER_H

#include <anymap.h>
#include <item.h>

typedef std::string LIB_NAME;
typedef std::string RETURN_TYPE;
typedef std::string TEMPLATE_LOADER_STRING;
typedef std::string FUNC_NAME;

class loader: public item{
private:
    LIB_NAME name;
    RETURN_TYPE ret_type;
    TEMPLATE_LOADER_STRING template_string;
    FUNC_NAME func;
public:
    loader(){
        name = "default_lib";
        ret_type = "HMODULE";
        func = "LoadLibrary";
    };
    loader(LIB_NAME libname, RETURN_TYPE return_type, FUNC_NAME func_name){
        name = libname;
        ret_type = return_type;
        func = func_name;
    };
    std::string get_comment() override{
        return "//Загрузка библиотеки " + name;
    }
    std::string serialize() override {
        return ret_type + " lib_" + name + "=" + func + "(L\""+ name + "\");";
    };
};


#endif //CODEGEN_PROJ_LOADER_H
