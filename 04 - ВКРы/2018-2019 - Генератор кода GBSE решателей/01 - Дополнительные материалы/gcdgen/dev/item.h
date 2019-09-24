//
// Created by semyon on 04.06.19.
//

#ifndef CODEGEN_ITEM_H
#define CODEGEN_ITEM_H

#include <string>

class item {
public:
    virtual std::string serialize() = 0;
    virtual std::string get_comment() = 0;
};


#endif //CODEGEN_ITEM_H
