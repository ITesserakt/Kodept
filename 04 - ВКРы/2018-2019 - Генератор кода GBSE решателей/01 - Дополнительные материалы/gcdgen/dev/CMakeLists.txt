cmake_minimum_required(VERSION 3.10.2)

project(programm_generator)

######################
### Исходные файлы ###
######################


########################################
### Добавление финальной цели сборки ###
########################################
set(SOURCES
        solvers.h
        caller.h
        caller.cpp
        loader.h
        loader.cpp
        getter.h
        getter.cpp
        programm_generator.h
        programm_generator.cpp
        )

add_library(${PROJECT_NAME} SHARED ${SOURCES})
target_link_libraries(${PROJECT_NAME} ${COMSDK_LIBRARY} ${Boost_LIBRARIES})

######################################################
### Добавление библиотеки в список на тестирование ###
######################################################

set(TESTED_LIBRARIES ${TESTED_LIBRARIES} ${PROJECT_NAME} CACHE STRING "" FORCE)
