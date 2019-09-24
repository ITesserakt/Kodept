//Класс-обертка над стандартным классом множества, адаптированный для работы с объектами loader, caller, getter

#ifndef CODEGEN_COLLECTION_H
#define CODEGEN_COLLECTION_H

#include <vector>
#include <string>
#include <algorithm>
#include <iostream>

template <typename T>
class collection {
private:
    std::vector<std::string> set;
public:
    collection(): set({}){};
    void add(std::string str){//Добавить строку в множество
        if((std::find(set.begin(), set.end(), str)==set.end())){
            set.push_back(str);
        }
    };
    void add(T item){
        auto str = item.serialize();
        auto comment = item.get_comment();
        if((std::find(set.begin(), set.end(), str)==set.end())){
                set.push_back(comment);
                set.push_back(str);
            }
    };
    void print(){
        for(int i = 0; i < set.size(); i++){
            std::cout<<set[i]<<" ";
        }
        std::cout<<std::endl;
    };
    void remove(int i){
        set.erase(set.begin()+i);
    }
    int size(){
        return set.size();
    }

    std::string operator[](int i){
        return set[i];
    }
};


#endif //CODEGEN_COLLECTION_H
