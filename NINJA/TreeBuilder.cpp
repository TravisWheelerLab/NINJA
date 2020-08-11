#include "TreeBuilder.hpp"

bool TreeBuilder::distInMem = false;
bool TreeBuilder::rebuildStepsConstant = false; // otherwise, decreasing
float TreeBuilder::rebuildStepRatio = (float) 0.5;
int TreeBuilder::rebuildSteps = -1;
int TreeBuilder::candidateIters = 50;
int TreeBuilder::verbose = 1;

TreeBuilder::TreeBuilder(std::string **names, int **distances, int namesSize) {
    this->firstActiveNode = 0; //not initialized in java, just to be sure

    this->nextActiveNode = nullptr;
    this->prevActiveNode = nullptr;


    this->names = names;
    this->D = distances;
    this->K = namesSize;

    this->R = new int[this->K]();

    this->redirect = new int[(this->K * 2) - 1];

    this->nodes = new TreeNode *[(this->K * 2) - 1];

    int i, j;

    for (i = 0; i < this->K; i++) {
        this->redirect[i] = i;
        this->nodes[i] = new TreeNode(names[i]);
    }

    for (i = this->K; i < 2 * this->K - 1; i++) {
        this->redirect[i] = -1;
        this->nodes[i] = new TreeNode();
    }

    for (i = 0; i < this->K - 1; i++) {
        for (j = i + 1; j < this->K; j++) {
            this->R[i] += (long) this->D[i][j - i - 1];
            this->R[j] += (long) this->D[i][j - i - 1];
        }
    }
}

TreeBuilder::~TreeBuilder() {
    delete[](this->R);
    delete[](this->redirect);
    delete[](this->nodes);

}

void TreeBuilder::finishMerging() { //did not change anything expect the ->`s instead of the .`s
    int last_i = 0, last_j = 0, last_k = 0;
    int ri, rj, rk;
    int i = 0;
    while (redirect[i++] == -1);
    ri = redirect[i - 1];
    last_i = i - 1;

    while (redirect[i++] == -1);
    rj = redirect[i - 1];
    last_j = i - 1;

    while (redirect[i++] == -1);
    rk = redirect[i - 1];
    last_k = i - 1;

    int nextNode = 2 * K - 3;

    nodes[nextNode]->leftChild = nodes[last_i];
    nodes[nextNode]->rightChild = nodes[last_j];

    int d_ij = ri < rj ? D[ri][rj] : D[rj][ri];

    nodes[last_i]->length = (d_ij + (R[ri] - R[rj]) / 2.0f) / 20000000.0f;
    nodes[last_j]->length = (d_ij + (R[rj] - R[ri]) / 2.0f) / 20000000.0f;

    //if a length is negative, move root of that subtree around to compensate.
    if (nodes[last_i]->length < 0) {
        nodes[last_j]->length -= nodes[last_i]->length;
        nodes[last_i]->length = 0;
    } else if (nodes[last_j]->length < 0) {
        nodes[last_i]->length -= nodes[last_j]->length;
        nodes[last_j]->length = 0;
    }


    int d_ik = ri < rk ? D[ri][rk] : D[rk][ri];
    int d_jk = rj < rk ? D[rj][rk] : D[rk][rj];
    float d_ijk = (d_ik + d_jk - d_ij) / 2.0f;

    nodes[nextNode + 1]->leftChild = nodes[nextNode];
    nodes[nextNode + 1]->rightChild = nodes[last_k];

    nodes[nextNode]->length = (d_ijk + (R[ri] - R[rj]) / 2.0f) / 20000000.0f;
    nodes[last_k]->length = (d_ijk + (R[rj] - R[ri]) / 2.0f) / 20000000.0f;

    //if a length is negative, move root of that subtree around to compensate.
    if (nodes[last_i]->length < 0) {
        nodes[last_j]->length -= nodes[last_i]->length;
        nodes[last_i]->length = 0;
    } else if (nodes[last_j]->length < 0) {
        nodes[last_i]->length -= nodes[last_j]->length;
        nodes[last_j]->length = 0;
    }
}

