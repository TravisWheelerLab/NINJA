/*
 * TreeBuilder.hpp
 *
 *  Created on: Jan 24, 2016
 *      Author: michel
 */
#ifndef TREEBUILDER_HPP
#define TREEBUILDER_HPP

#include <string>
#include <vector>
#include "TreeNode.hpp"



class TreeBuilder{
	public:
		TreeBuilder (std::string** names, int** distances, int namesSize);
		~TreeBuilder();
		TreeNode* build ();

		//revisit this, it is static in the java program
		static bool distInMem;
		static bool rebuildStepsConstant;
		static float rebuildStepRatio;
		static int rebuildSteps;
		static const int clustCnt = 30;
		static int candidateIters;
		static int verbose;
		int K;

	protected:
		std::string** names;
		int** D;
		long int *R;
		TreeNode **nodes;
		int *redirect;
		int *nextActiveNode;
		int *prevActiveNode;
		int firstActiveNode;

		void finishMerging();

};

#endif
