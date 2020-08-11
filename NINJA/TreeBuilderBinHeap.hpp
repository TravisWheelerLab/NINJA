#include "BinaryHeap_IntKey_TwoInts.hpp"
#include "Stack.hpp"
#include "TreeBuilder.hpp"

#include <climits>

class TreeBuilderBinHeap : public TreeBuilder {
public:
    TreeBuilderBinHeap(std::string **names, int **distances, int namesSIze);

    ~TreeBuilderBinHeap();

    int *candidateCountPerLoop;

    TreeNode **build();

private:
    BinaryHeap_IntKey_TwoInts ***heaps;

    int *clustAssignments;
    long *clustPercentiles;
    int *clustersBySize;
    int *clustSizes;

    Int candidatesD;
    Int candidatesI;
    Int candidatesJ;
    bool *candidatesActive;
    Stack *freeCandidates;
    int lastCandidateIndex;

    void clusterAndHeap(int maxIndex);

    int appendCandidate(int d, int i, int j);
};


