#ifndef TREEBUILDER_HPP
#define TREEBUILDER_HPP

#include "TreeNode.hpp"

#include <string>
#include <vector>

class TreeBuilder {
public:
    TreeBuilder(std::string **names, int **distances, int namesSize);

    ~TreeBuilder();

    static bool distInMem;
    static bool rebuildStepsConstant;
    static float rebuildStepRatio;
    static int rebuildSteps;
    static int candidateIters;
    static int verbose;
    int K;

protected:
    std::string **names;
    int **D;
    int *R;
    TreeNode **nodes;
    int *redirect;
    int *nextActiveNode;
    int *prevActiveNode;
    int firstActiveNode;

    void finishMerging();
};

#endif
