#include "BinaryHeap.hpp"
#include "ExceptionHandler.hpp"

#include <climits>

struct ints3 {
    int first;
    int second;
    int key;
};

class BinaryHeap_IntKey_TwoInts {
public:
    BinaryHeap_IntKey_TwoInts();

    BinaryHeap_IntKey_TwoInts(int maxCapacity);

    BinaryHeap_IntKey_TwoInts(const int *val1s, const int *val2s, Int keys);

    BinaryHeap_IntKey_TwoInts(const int *val1s, const int *val2s, Int keys, int maxCapacity);

    ~BinaryHeap_IntKey_TwoInts();

    int size() const; //return BinaryHeap size

    //functions
    void insert(int val1, int val2, int key) const;

    void deleteMin() const; //Remove the smallest item from the priority queue

    bool isEmpty() const; //check if empty
    void makeEmpty() const; //empty heap

    std::vector<ints3> *heap;


    static const int DEFAULT_CAPACITY = 1000;

    static bool binHeapTwoTest(bool verbose);

};
