cmake_minimum_required(VERSION 2.8)

project(jugr)

######################
### Исходные файлы ###
######################


########################################
### Добавление финальной цели сборки ###
########################################

set(SOURCES
      jugr.cpp
      )

add_library(${PROJECT_NAME} SHARED ${SOURCES})
target_link_libraries(${PROJECT_NAME} ${COMSDK_LIBRARY} ${Boost_LIBRARIES})


######################################################
### Добавление библиотеки в список на тестирование ###
######################################################

set(TESTED_LIBRARIES ${TESTED_LIBRARIES} ${PROJECT_NAME} CACHE STRING "" FORCE)
