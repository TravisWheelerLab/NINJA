#include "Stack.hpp"

Stack::Stack() {
    this->mystack = new std::stack<int>();
}

Stack::~Stack() {
    delete this->mystack;
}

void Stack::clear() {
    delete this->mystack;
    this->mystack = new std::stack<int>();

}

int Stack::length() {
    return this->mystack->size();
    this->mystack = new std::stack<int>();
}

void Stack::push(int x) {
    this->mystack->push(x);
}

int Stack::pop() {
    int ret = this->mystack->top();
    this->mystack->pop();
    return ret;
}

bool Stack::isEmpty() {
    return this->mystack->empty();
}
