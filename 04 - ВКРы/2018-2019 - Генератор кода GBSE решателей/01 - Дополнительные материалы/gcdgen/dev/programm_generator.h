//
// Created by semyon on 20.05.19.
//
//TODO: ДОБАВИТЬ КЛАСС МНОЖЕСТВА И ПРОВЕРЯТЬ НА УНИКАЛЬНОСТЬ
//TODO: подумать над тем, как в множестве обрабатывать пары и строки без дублирования кода
//TODO: научиться забирать данные из Anymap

#ifndef CODEGEN_PROJ_PROGRAMM_GENERATOR_H
#define CODEGEN_PROJ_PROGRAMM_GENERATOR_H

#include <vector>
#include <string>
#include <fstream>
#include <anymap.h>
#include <cycle.h>
#include <collection.h>
#include "loader.h"
#include "getter.h"
#include "caller.h"


std::string default_inject_header =
        "#include <anymap.h>\ntypedef int processorFuncType(AnyMap& );\n"
        "typedef bool predicateFuncType(const AnyMap& );\n"
        "template<processorFuncType* tf, predicateFuncType* tp>\n"
        "int F(AnyMap& p_m)\n"
        "{\n"
        "\treturn (tp(p_m))?tf(p_m):tp(p_m);\n"
        "}\nint main(){\n\t";

std::string default_proc_type = "processorFuncType *";
std::string default_pred_type = "predicateFuncType *";
std::string default_loader = "LoadLibrary";
std::string default_search_func = "GetProcAddress";
std::string input_aini_name =  "input";
std::string default_inject_footer =
        "\treturn 0;\n}";

typedef std::string HEADERS;
typedef std::string FOOTER;
typedef std::vector<std::vector<std::string>> call_stack;
typedef std::string filename;
typedef std::string programm_name;
class programm_generator {
private:
    programm_name name;
    HEADERS header;//#include ..,  определения функций(если нужно), int main() {
    FOOTER footer;//return 0;} и информация о дате создания
    com::Anymap data_ini;//ini объект
    collection<loader> loader_set;
    collection<getter> getter_set;
    collection<caller> caller_set;
    std::string cycleForOutput;
public:
    programm_generator() {
        header = default_inject_header;
        footer = default_inject_footer;
        loader_set = {};
        getter_set = {};
        caller_set = {};
    };

    programm_generator(com::Anymap& inifile, call_stack stack){
        for(int i = 0; i < stack.size(); i++){
            for(int j = 0; j < stack[i].size(); j++){
                std::cout<<stack[i][j]<<" ";
            }
            std::cout<<std::endl;
        }
        std::cout<<std::endl;
        header = default_inject_header;
        footer = default_inject_footer;
        data_ini = inifile;
        auto items = splitVector(stack);

        for (int i = 0; i < stack.size(); i++){
            if (stack[i][stack[i].size() - 1] == "cycle"){
                std::vector<std::string> stringsForCycle;
                std::string condition;
                int j = 0;
                for (j = 0; j < items.first.size(); j++) {
                    std::string pred;
                    if (items.first[j][2] != "0") {
                        pred = items.first[j][2];
                    } else {
                        pred = "default_pred_name";
                    }
                    //Грузить библиотеку из aini, пока что заплатка
                    getter tmp_getter(default_proc_type, default_pred_type, items.first[j][1], pred, default_search_func, "lib_name");
                    loader tmp_loader("lib_name", "HMODULE", default_loader);
                    caller tmp_caller(items.first[j][1], pred, "auto", "F", "res", input_aini_name);
                    stringsForCycle.push_back(tmp_caller.get_comment());
                    stringsForCycle.push_back(tmp_caller.serialize());
                    loader_set.add(tmp_loader.get_comment());
                    loader_set.add(tmp_loader.serialize());
                    getter_set.add(tmp_getter.get_comment());
                    getter_set.add(tmp_getter.serialize().first);
                    getter_set.add(tmp_getter.serialize().second);
                }
                i += j;
                condition = stack[i][2] == "True"? stack[i][2]: "!" + stack[i][2];
                std::cout<<"hui"<<std::endl;
                //condition = "True";
                cycle cyc(stringsForCycle, condition);
                caller_set.add(cyc.serialize());
            } else {
                    std::string pred;
                    if(stack[i][2] != "0") {
                        pred = stack[i][2];
                    } else {
                        pred = "default_pred_name";
                    }
                    //Грузить библиотеку из aini, пока что заплатка
                    std::cout<<pred<<" "<<stack[i][1]<<std::endl;
                    getter tmp_getter(default_proc_type, default_pred_type, stack[i][1], pred, default_search_func, "lib_name");
                    loader tmp_loader("lib_name", "HMODULE", default_loader);
                    caller tmp_caller(stack[i][1], pred, "auto", "F", "res", input_aini_name);
                    getter_set.add(tmp_getter.get_comment());
                    getter_set.add(tmp_getter.serialize().first);
                    getter_set.add(tmp_getter.serialize().second);
                    loader_set.add(tmp_loader);
                    caller_set.add(tmp_caller);
            }
        }

       /* for(int i = 0; i < items.second.size(); i++){
            std::string pred;
            if(items.second[i][2] != "0") {
                pred = items.second[i][2];
            } else {
                pred = "default_pred_name";
            }
            //Грузить библиотеку из aini, пока что заплатка
            std::cout<<pred<<" "<<items.first[i][1]<<std::endl;
            getter tmp_getter(default_proc_type, default_pred_type, items.second[i][1], pred, default_search_func, "lib_name");
            loader tmp_loader("lib_name", "HMODULE", default_loader);
            caller tmp_caller(items.second[i][1], pred, "auto", "F", "res", input_aini_name);
            getter_set.add(tmp_getter.get_comment());
            getter_set.add(tmp_getter.serialize().first);
            getter_set.add(tmp_getter.serialize().second);
            loader_set.add(tmp_loader);
            caller_set.add(tmp_caller);
        }*/
        /*if(items.first.size() != 0) {
            std::vector<std::string> stringsForCycle;
            std::string condition;
            for (int i = 0; i < items.first.size(); i++) {
                std::string pred;
                if (items.first[i][2] != "0") {
                    pred = items.first[i][2];
                } else {
                    pred = "default_pred_name";
                }
                //Грузить библиотеку из aini, пока что заплатка
                getter tmp_getter(default_proc_type, default_pred_type, items.first[i][1], pred, default_search_func, "lib_name");
                loader tmp_loader("lib_name", "HMODULE", default_loader);
                caller tmp_caller(items.first[i][1], pred, "auto", "F", "res", input_aini_name);
                stringsForCycle.push_back(tmp_caller.get_comment());
                stringsForCycle.push_back(tmp_caller.serialize());
                loader_set.add(tmp_loader.get_comment());
                loader_set.add(tmp_loader.serialize());
                getter_set.add(tmp_getter.get_comment());
                getter_set.add(tmp_getter.serialize().first);
                getter_set.add(tmp_getter.serialize().second);
            }
            cycle cyc(stringsForCycle, "1");
            cycleForOutput = cyc.serialize();
        }*/
    }
    std::pair<std::vector<std::vector<std::string>>, std::vector<std::vector<std::string>>> splitVector(call_stack stack){
        std::vector<std::vector<std::string>> notInCycle;
        std::vector<std::vector<std::string>> inCycle;
        for(int i = 0; i < stack.size(); i++){
            if(stack[i][stack[i].size()-1] == "cycle"){
                inCycle.push_back(stack[i]);
            }
            else {
                notInCycle.push_back(stack[i]);
            }
        }
        return std::make_pair(inCycle, notInCycle);
    }
    programm_generator(HEADERS inject_header, filename inifile,
                       call_stack stack, FOOTER inject_footer) {
        header = inject_header;
        footer = inject_footer;
        /* обработка стека вызова - получение всех функций и создание объектов caller, getter,  loader*/
        /*for(int i = 0; i < stack.size(); i++){
            if (stack[i][1] == "0"){
                getter tmp(, stack[i][1], );
            }
        }*/
    };
    ~programm_generator(){};
    std::ofstream serialize(filename outputData) {
        std::ofstream out(outputData);
        out<<header;
        for(int i = 0; i < loader_set.size(); i++){
            out<<"\t"<<loader_set[i]<<"\n";
        }
        for(int i = 0; i < getter_set.size(); i++){
            out<<"\t"<<getter_set[i]<<"\n";
        }
        for(int i = 0; i < caller_set.size(); i++){
            out<<"\t"<<caller_set[i]<<"\n";
        }
        out<<cycleForOutput<<"\n";
        out<<footer<<"\n";
        return out;
    };
};


#endif //CODEGEN_PROJ_PROGRAMM_GENERATOR_H
