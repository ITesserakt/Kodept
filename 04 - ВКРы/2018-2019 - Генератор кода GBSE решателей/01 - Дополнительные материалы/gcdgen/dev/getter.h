//Класс генерирующий строки следующего вида и комментарии к ним:
//processorFuncType *proc_@processor_i@ = (processorFuncType*)GetProcAddress(lib_@lib_name_i@, "@processor_i@");


#ifndef CODEGEN_PROJ_GETTER_H
#define CODEGEN_PROJ_GETTER_H

#include <string>

typedef std::string PROC_FUNC_TYPE;
typedef std::string PRED_FUNC_TYPE;
typedef std::string PROC_FUNC_NAME;
typedef std::string PRED_FUNC_NAME;
typedef std::string SEARCH_FUNC_NAME;
typedef std::string LIB_TO_SEARCH;

class getter {
private:
    PROC_FUNC_TYPE procFuncType;
    PRED_FUNC_TYPE predFuncType;
    PROC_FUNC_NAME procFuncName;
    PRED_FUNC_NAME predFuncName;
    SEARCH_FUNC_NAME searchFuncName;
    LIB_TO_SEARCH libToSearch;
public:
    getter(){
        predFuncType = "default_predicate *";
        procFuncType = "default_processor *";
        procFuncName = "def_proc_name";
        predFuncName = "def_pred_name";
        searchFuncName = "GetProcAddress";
        libToSearch = "default_lib";
    };
    getter(PROC_FUNC_TYPE proc_func_type, PRED_FUNC_TYPE pred_func_type,
            PROC_FUNC_NAME proc_func_name,PRED_FUNC_NAME pred_func_name, SEARCH_FUNC_NAME search_func_name, LIB_TO_SEARCH lib){
        procFuncType = proc_func_type;
        predFuncType = pred_func_type;
        procFuncName = proc_func_name;
        predFuncName = pred_func_name;
        searchFuncName = search_func_name;
        libToSearch = lib;
    }
    std::string get_comment(){
        return "//Поиск функции обработчика " + procFuncName + " и функции-предиката " + predFuncName + " в библиотеке " +  libToSearch;
    };
    std::pair<std::string, std::string> serialize(){
        auto processor = procFuncType + "proc_" + procFuncName + " = " +
                "(" + procFuncType + ")" + searchFuncName + "(" + libToSearch + ",\"" + procFuncName + "\");";
        auto predicate = predFuncType + "pred_" + predFuncName + " = " +
                         "(" + predFuncType + ")" + searchFuncName + "(" + libToSearch + ", \"" + predFuncName + "\");";
        auto returnSet = std::pair<std::string, std::string>(processor, predicate);
        return returnSet;
    };

};


#endif //CODEGEN_PROJ_GETTER_H
