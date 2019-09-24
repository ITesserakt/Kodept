//
// Created by semyon on 20.05.19.
//
#include <solvers.h>
#include <getter.h>
#include <programm_generator.h>
#include <loader.h>
#include <caller.h>
#include <cycle.h>
#include "item.h"
#include "collection.h"
#include "graph.h"
#include <anymap.h>
#include <iniparser.h>
#include <iostream>
#include "edge.h"
#include "dynfunction_executor.h"
#include "filetools.h"
#include "stringtools.h"
#include "logger.h"
#include "libtools.h"
#include "logger.h"
#include "plain_context.h"
#include "action_item_context.h"
#include <functional>
#include <utility>
#include <typeinfo>
using namespace com::sys;
using namespace com::lib;
using namespace std;
using namespace com;
using namespace com::graph;
using namespace com::fs;
using namespace com::str;
using namespace ini;
using namespace std;


int checkCycle(std::vector<std::vector<string>> functionList, std::vector<detail::NodeHelper*> childrenList, int inputID){
    for(auto itChildren = childrenList.begin(); itChildren != childrenList.end(); itChildren++){
        auto tmpChildren = *itChildren;
        auto tmp = to_string(tmpChildren->id);
        for(auto it = functionList.begin(); it != functionList.end(); it++){
            if((*(*it).begin() == tmp)){
                return atoi(tmp.c_str());
            }
        }
    }
    return -1;
}

std::vector<std::vector<std::string>> runWithoutExec(const std::string& adotFile)
{
    //строим модель графа
    map<std::string, detail::NodeHelper*> nodesLookup;
    map<std::string, map<std::string, std::string> > funcLookup;
    std::vector<std::vector<std::string>> functionList;
    detail::NodeHelper* entryNode;
    ifstream fs(adotFile.c_str());
    string line;
    bool insideBody = false;
    while (fileGetLine(fs, line)) {
        line = trim(line);
        if (line.empty()) {
            continue;
        }

        if (!insideBody) { // тело начинается там, где '{'
            if (line.find('{') != string::npos) {
                insideBody = true;
                continue;
            }
        }
        else { // конец тела тогда, когда '}'
            if (line.find('}') != string::npos) {
                break;
            }
            parseDotLine(line, nodesLookup, funcLookup, entryNode);
        }
    }


    for (auto itNode = nodesLookup.begin(); itNode != nodesLookup.end(); ++itNode) {
        bool flag = 0;
        std::vector<std::string> func;
       // std::cout<<toString(itNode->second->id)<<" ";
       // std::cout<<toString(itNode->second->name)<<" ";
        int inputId;
        int finishId;
        if(itNode->second->name == "INPUT"){
            inputId = itNode->second->id;
        }
        for(int i = 0; i < itNode->second->children.size(); i++){
            if(itNode->second->children[i]->name == "FINILEZED"){
                finishId = itNode->second->id;
            }
            //std::cout<<toString(itNode->second->children[i]->id)<<" ";
        }
        for (auto itNodeEdges = itNode->second->edges.begin();
             itNodeEdges != itNode->second->edges.end(); itNodeEdges++){
            func.push_back(toString(itNode->second->id));
            func.push_back(funcLookup[itNodeEdges->props["edge"]]["entry_func"]);
            func.push_back(toString(0));
            func.push_back(toString(0));
            if(itNodeEdges->props.count("on_predicate_value")!=0){
                func[2] = funcLookup[ itNode->second->props["predicate"]]["entry_func"];
                func[3] = toString(itNodeEdges->props["on_predicate_value"]);
            }

            /* if(!flag){
                 flag = 1;
                 for(int i = 0; i < itNode->second->children.size(); i++) {
                     func.push_back(toString(itNode->second->children[i]->name));
                 }
             }*/
            functionList.push_back(func);
            func.clear();
        }
        int posStartCycle;
        if(((checkCycle(functionList, itNode->second->children,inputId))!=-1) && itNode->second->id!=inputId){
            posStartCycle = checkCycle(functionList, itNode->second->children,inputId);
            /*    for(auto it = functionList.begin(); it != functionList.end(); it++){
                     if(*(*it).begin() == to_string(pos)){
                         std::cout<<*(*it).begin();
                         }
                     }*/
        }


        for (int i = posStartCycle; i < functionList.size(); i++) {
            std::cout<<toString(functionList[i][0])<<" "<<toString(i)<<std::endl;
            do {
                if( atoi(functionList[i][0].c_str()) != inputId && (!(atoi(functionList[i][0].c_str()) == finishId && functionList[i][3] != "false")))
                    functionList[i].push_back("cycle");
                if (toString(functionList[i][0]) != toString(i))
                    break;
            } while(i++);
        }
    }
    return functionList;
}

std::vector<std::vector<std::string>> test_call_stack(){
    auto call_stack = runWithoutExec("graph_jugr_test.dot");
    std::swap(call_stack[2], call_stack[0]);
    return call_stack;
}

std::vector<std::vector<std::string>> sortForId(std::vector<std::vector<std::string>> sequence){
    //TODO: реализовать алгоритм сортировки для последовательности, пока костыль
}

void loaderDefaultTest(){
    loader load_default;
    auto comment = load_default.get_comment();
    auto str = load_default.serialize();
    std::cout<<"Тест 1: Проверка строки загрузки библиотек"<<std::endl;
    std::cout<<comment<<std::endl<<str<<std::endl;
}

void callerDefaultTest(){
    caller caller_default;
    auto comment = caller_default.get_comment();
    auto str = caller_default.serialize();
    std::cout<<"Тест 2: Проверка строк поиска функций в библиотеках"<<std::endl;
    std::cout<<comment<<std::endl<<str<<std::endl;
}

void getterDefaultTest(){
    getter getter_default;
    auto comment = getter_default.get_comment();
    auto str = getter_default.serialize();
    std::cout<<"Тест 3: Проверка строки запуска функции"<<std::endl;
    std::cout<<comment<<std::endl<<str.first<<std::endl<<str.second<<std::endl;
}

void collectionTest() {
    std::cout<<"Тест 4: Проверка работы коллекции с loader строками\n";
    loader load;
    collection<loader> coll;
    std::cout<<"В коллекцию добавляется строка полученная явным вызовом serialize:\n";
    coll.add(load.serialize());
    coll.print();
    std::cout<<"В коллекцию добавляется строка полученная loader'ом:\n";
    coll.add(load);
    coll.print();
    std::cout<<"Из коллекции удаляется строка:\n";
    coll.remove(0);
    coll.print();
    std::cout<<"В коллекцию добавляется строка полученная loader'ом:\n";
    coll.add(load);
    coll.print();
}

void cycleTest(){
    std::cout<<"Тест 5: Цикл"<<std::endl;
    cycle cyc1({"a += 1"}, " a < 3 ");
    std::cout<<cyc1.serialize()<<std::endl;
}

int main(){
  /*  loaderDefaultTest();
    callerDefaultTest();
    getterDefaultTest();*/
    collectionTest();
    cycleTest();
    auto queue = test_call_stack();
    com::Anymap input("input.aini");
    std::cout<<input["a"]<<std::endl;
    programm_generator pg(input, queue);
    pg.serialize("tst.cpp");
    return 0;
}