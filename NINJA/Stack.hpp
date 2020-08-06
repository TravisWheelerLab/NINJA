#ifndef STACK_HPP
#define STACK_HPP

#include "ExceptionHandler.hpp"

#include <cstring>
#include <stack>

class Stack {
public:
    Stack();

    ~Stack();

    void push(int x);

    int pop();

    int length();

    bool isEmpty();

    void clear();

private:
    std::stack<int> *mystack;
};

#endif
