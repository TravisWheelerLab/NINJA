#ifndef BINARYHEAP_HPP
#define BINARYHEAP_HPP

#include "ExceptionHandler.hpp"

#include <cassert>
#include <climits>
#include <cstdio>

struct Int {
    int *pointer;
    int length;
};
struct Float {
    float *pointer;
    int length;
};

class BinaryHeap {
public:
    //constructors
    BinaryHeap();

    BinaryHeap(int maxCapacity);

    BinaryHeap(const int *val1s, Int keys);

    BinaryHeap(const int *val1s, Int keys, int maxCapacity);

    ~BinaryHeap();


    int size() const; //return BinaryHeap size

    //functions
    int insert(int val1, int key) const; //insert element
    void deleteMin() const; //Remove the smallest item from the priority queue

    bool isEmpty() const; //check if empty
    void makeEmpty() const; //empty heap

    static bool binHeapTest(bool verbose);

    std::vector<std::pair<int, int> > *heap;

    static const int DEFAULT_CAPACITY = 1000;
};

#endif
