cmake_minimum_required(VERSION 3.21)
project(nettool_qt LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Widgets Network)

add_executable(nettool-qt
    src/main.cpp
    src/MainWindow.cpp
    src/MainWindow.h
    src/Translator.cpp
    src/Translator.h
    src/Dialogs.cpp
    src/Dialogs.h
    src/Tools.cpp
    src/Tools.h
    src/PacketEditor.cpp
    src/PacketEditor.h
    src/NotificationCenter.cpp
    src/NotificationCenter.h
    src/WebDashboard.cpp
    src/WebDashboard.h
)

target_link_libraries(nettool-qt PRIVATE Qt6::Widgets Qt6::Network)

add_custom_command(TARGET nettool-qt POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
        ${CMAKE_CURRENT_SOURCE_DIR}/lang
        $<TARGET_FILE_DIR:nettool-qt>/lang
)

if(MSVC)
    target_compile_options(nettool-qt PRIVATE /utf-8)
endif()
