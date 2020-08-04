#ifndef BINARYHEAP_TWO_INTS_HPP
#define BINARYHEAP_TWO_INTS_HPP

#include <climits>

#include "ExceptionHandler.hpp"
#include "BinaryHeap.hpp"

struct ints2float {
    int first;
    int second;
    float key;
};

class BinaryHeap_TwoInts {
public:
    //constructors
    BinaryHeap_TwoInts();

    BinaryHeap_TwoInts(int maxCapacity);

    BinaryHeap_TwoInts(const int *val1s, const int *val2s, Float keys);

    BinaryHeap_TwoInts(const int *val1s, const int *val2s, Float keys, int maxCapacity);

    ~BinaryHeap_TwoInts();


    int size() const; //return BinaryHeap size

    //functions
    void insert(int val1, int val2, float key) const;

    void deleteMin() const; //Remove the smallest item from the priority queue

    bool isEmpty() const; //check if empty
    void makeEmpty() const; //empty heap
    void buildAgain(const int *val1s, const int *val2s, Float keys);

    void chopBottomK(int *val1s, int *val2s, Float keys) const;

    std::vector<ints2float> *heap;

    static const int DEFAULT_CAPACITY = 1000;

    static bool binHeapTwoTest(bool verbose);

};

#endif
