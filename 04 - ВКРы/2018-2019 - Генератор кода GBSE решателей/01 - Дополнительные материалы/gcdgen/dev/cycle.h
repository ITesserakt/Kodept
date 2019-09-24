//Класс генерирующий цикл на основе нескольких входных строк

#ifndef CODEGEN_CYCLE_H
#define CODEGEN_CYCLE_H

#include <string>
#include <vector>
#include <item.h>

typedef std::vector<std::string> REPEATE_SET;
typedef std::string CONDITION;

class cycle: public item {
private:
    REPEATE_SET repeate_set;
    CONDITION cond;
public:
    cycle():repeate_set({}), cond("1"){};
    cycle(CONDITION condition): repeate_set({}), cond(condition){};
    cycle(REPEATE_SET rep_set):repeate_set(rep_set), cond("1"){};
    cycle(REPEATE_SET  rep_set, CONDITION condition): repeate_set(rep_set), cond(condition){};
    std::string serialize(){
        std::string repeats = "\tdo {\n";
        for(int i = 0; i < repeate_set.size(); i++){
            repeats += "\t";
            repeats += repeate_set[i];
            repeats += "\n";
        }
        repeats += "} while(" + cond + ");\n";
        return  repeats;
    }
    std::string get_comment(){
        return "//cycle\n";
    }
};


#endif //CODEGEN_CYCLE_H
