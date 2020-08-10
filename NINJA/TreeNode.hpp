#ifndef TREENODE_HPP
#define TREENODE_HPP

#include <cfloat>
#include <cstdio>
#include <cstdlib>
#include <string>

class TreeNode {
public:
    TreeNode();

    TreeNode(std::string *name);

    ~TreeNode();

    TreeNode *leftChild = nullptr;
    TreeNode *rightChild = nullptr;
    std::string *name;
    float length = FLT_MAX;

    void buildTreeString(std::string *sb);
};

#endif
