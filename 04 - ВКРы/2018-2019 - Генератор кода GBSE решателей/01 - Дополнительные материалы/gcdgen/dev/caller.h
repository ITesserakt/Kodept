//Класс генерирующий строки следующего вида и комментарии к ним:
//res = F<proc_@processor_i@,pred_@predicate_i@>(m); if (res != 0) return res;

#ifndef CODEGEN_PROJ_CALLER_H
#define CODEGEN_PROJ_CALLER_H

#include <string>
#include <item.h>

typedef std::string PROCESSOR;//Функция - обработчик
typedef std::string PREDICATE;//Функция предикат
typedef std::string RETURN_VALUE_TYPE;//Тип возвращаемого функцией значения
typedef std::string FUNC_NAME;//Имя вызываемой функции
typedef std::string VARIABLE_NAME;//Имя переменной, в которую будет помещено возвращаемое значение
typedef std::string ANYMAP_NAME;//Имя Anymap

//Класс создания строки вызова функции
class caller: public item {
private:
    PROCESSOR proc;
    PREDICATE pred;
    RETURN_VALUE_TYPE retValue;
    FUNC_NAME functionName;
    VARIABLE_NAME variableName;
    ANYMAP_NAME anymapName;
public:
    caller(){
        proc = "default_processor";
        pred = "default_predicate";
        retValue = "auto";
        functionName = "F";
        variableName = "res";
        anymapName = "default_anymap";
    }
    caller(PROCESSOR processor, PREDICATE predicate, RETURN_VALUE_TYPE returnValueType,
            FUNC_NAME funcName, VARIABLE_NAME var, ANYMAP_NAME anymap){
        proc = processor;
        pred = predicate;
        retValue = returnValueType;
        functionName = funcName;
        variableName = var;
        anymapName = anymap;
    };

    std::string get_comment(){
        return "//Вызов функции-обработчика " + proc + " с предикатом " + pred;

    };

    std::string serialize(){
        return retValue + " " + variableName + " = " + functionName +
            "<" + "proc_" + proc + ", " + "pred_" + pred + ">(" + anymapName + ");if (" + variableName + "!=0) return " + variableName + ";";
    };
};


#endif //CODEGEN_PROJ_CALLER_H
