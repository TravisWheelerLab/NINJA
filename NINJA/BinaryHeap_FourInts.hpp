#ifndef BINARYHEAP_FOUR_INTS_HPP
#define BINARYHEAP_FOUR_INTS_HPP

#include "BinaryHeap.hpp"
#include "ExceptionHandler.hpp"

#include <climits>

struct ints4float {
    int first;
    int second;
    int third;
    int fourth;
    float key;
};

class BinaryHeap_FourInts {
public:
    //constructors
    BinaryHeap_FourInts();

    BinaryHeap_FourInts(int maxCapacity);

    BinaryHeap_FourInts(const int *val1s, const int *val2s, const int *val3s, const int *val4s, Float keys);

    BinaryHeap_FourInts(const int *val1s, const int *val2s, const int *val3s, const int *val4s, Float keys, int maxCapacity);

    ~BinaryHeap_FourInts();


    int size() const; //return BinaryHeap size

    //functions
    void insert(int val1, int val2, int val3, int val4, float key) const;

    void deleteMin() const; //Remove the smallest item from the priority queue

    bool isEmpty() const; //check if empty
    void makeEmpty(); //empty heap

    void buildAgain(const int *val1s, const int *val2s, const int *val3s, const int *val4s, Float keys);

    std::vector<ints4float> *heap;

    static const int DEFAULT_CAPACITY = 1000;

    static bool binHeapFourTest(bool verbose);
};

#endif
