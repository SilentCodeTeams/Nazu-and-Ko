cmake_minimum_required(VERSION 3.21)
project(nettool_qt LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Widgets)

add_executable(nettool-qt
    src/main.cpp
    src/MainWindow.cpp
    src/MainWindow.h
)

target_link_libraries(nettool-qt PRIVATE Qt6::Widgets)

if(MSVC)
    target_compile_options(nettool-qt PRIVATE /utf-8)
endif()
