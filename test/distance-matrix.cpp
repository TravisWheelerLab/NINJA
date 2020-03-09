//
// Created by sarah on 3/6/20.
//

#include "gtest/gtest.h"
#include "DistanceCalculator.hpp"
#include "TreeBuilderManager.hpp"
#include "ArgumentHandler.hpp"
#include "BinaryHeap.hpp"
#include <math.h>
#include <float.h>
#include <stdint.h>
#include <string>
#include <iostream>
#include <fstream>
#include <algorithm>
#include <map>

int gaps = 0;
int ts1_3 = 0;  //counter to keep track of transitions between seq2 and seq3
int tv1_3 = 0;  //counter to keep track of transversions between seq2 and seq3
const int numSeqs = 3;

static const char alphabet[] =
        "A"
        "T"
        "C"
        "G";

TEST(DistCalcTest, DistMatrix10) {
    char *argv[] = {"../build/src/NINJA_run", "--in", "../src/fixtures/10.fa", "--out_type", "d", "--corr_type", "k"};
    ArgumentHandler *argHandler = new ArgumentHandler(argv,8);

    std::string method = argHandler->getMethod();
    std::string inFile = argHandler->getInFile();
    std::string njTmpDir = argHandler->getNJTmpDir();
    ArgumentHandler::InputType inType = argHandler->getInputType();
    ArgumentHandler::OutputType outType = argHandler->getOutputType();
    ArgumentHandler::AlphabetType alphType = argHandler->getAlphabetType();
    ArgumentHandler::CorrectionType corrType = argHandler->getCorrectionType();
    FILE* out = argHandler->getOutpuFile();
    int threads = argHandler->getNumThreads();
    bool useSSE = argHandler->useSSE();
    bool printTime = argHandler->getPrintTime();

    TreeBuilderManager* manager = new TreeBuilderManager(method, njTmpDir, inFile, out, (TreeBuilderManager::InputType)
            inType,(TreeBuilderManager::OutputType) outType, (TreeBuilderManager::AlphabetType)
                                                                 alphType,(TreeBuilderManager::CorrectionType) corrType, threads, useSSE, printTime);

    std::string treeString = manager->doJob();


    //now do random generation for comparison!
    const unsigned long length = 10;
    char seq1[length] = {};
    char seq2[length] = {};
    char seq3[length] = {};
    int allowed_mutations = 2;
    std::ofstream outfile("../src/fixtures/10-test.fa",std::ios::out);
    //TODO: call random_generator()
}


//returns an index after checking the secondary map to make sure it hasn't already been used
int find_index(int length, std::map<int, bool> used){
    int index = rand() % length;
    auto search = used.find(index);
    if (search->second != true) {
        return index;
    }
    else {
        index = find_index(length, used);
    }
}

/*Because each nucleotide has multiple choices for transversions,
* this function randomly chooses between the two "paths" and then
* returns a different character to induce that transversion*/
unsigned char transversion(char nuc) {
    int num = rand() % 2;
    if(num == 0){
        if (nuc == 'G')
            return 'C';
        if (nuc == 'C')
            return 'G';
        if (nuc == 'T')
            return 'A';
        if (nuc == 'A')
            return 'T';
    }

    if(num == 1){
        if (nuc == 'G')
            return 'T';
        if (nuc == 'C')
            return 'A';
        if (nuc == 'T')
            return 'G';
        if (nuc == 'A')
            return 'C';
    }

    else{
        gaps++;
        return '-';
    }
}

/*Transitions are much simpler, so this just returns the correct nucleotide for
* inducing a transition in a given nucleotide*/
unsigned char transition(char nuc) {
    //A-G   C-T
    if (nuc == 'G')
        return 'A';
    if (nuc == 'C')
        return 'T';
    if (nuc == 'T')
        return 'C';
    if (nuc == 'A')
        return 'G';

    return -1;
}

void check_index(char nuc1, char nuc2){
    if(nuc1 == nuc2){
        return;
    }
    else if(nuc1 == 'G'){
        if(nuc2 == 'A')
            ts1_3++;
        else if(nuc2 == 'C' || nuc2 == 'T')
            tv1_3++;
        return;
    }

    else if (nuc1 == 'C') {
        if (nuc2 == 'T')
            ts1_3++;
        else if (nuc2 == 'G' || nuc2 == 'A')
            tv1_3++;
        return;
    }
    else if (nuc1 == 'T'){
        if(nuc2 == 'C')
            ts1_3++;
        else if(nuc2 == 'A' || nuc2 == 'G')
            tv1_3++;
        return;
    }

    else if (nuc1 == 'A'){
        if(nuc2 == 'G')
            ts1_3++;
        else if(nuc2 == 'T' || nuc2 == 'C')
            tv1_3++;
        return;
    }
}

double** random_generator(char seq1[], char seq2[], char seq3[], unsigned long length, int allowed_mutations, std::ofstream outfile){
    const unsigned int transitions = rand() % allowed_mutations;
    const unsigned int transversions = rand() % allowed_mutations;
    const unsigned int transitions2 = rand() % allowed_mutations;
    const unsigned int transversions2 = rand() % allowed_mutations;

    std::string names[numSeqs] = {"Seq1", "Seq2", "Seq3"};


    //this is a secondary structure used to just denote when a given index has already been
    //chosen as a spot to induce a transition/transversion - no "double dipping" the indices
    //gives us a guarantee that the given numbers to induce are truly in the output sequences
    //there is one for seq2 and seq3 - seq1 doesn't need one as no mutations are induced in it
    std::map<int, bool> usedSeq2;
    std::map<int, bool> usedSeq3;


    //this creates seq1 and seq2
    for(int i = 0; i < length; i++){
        char letter = alphabet[rand() % 4];
        seq1[i] = letter;
        seq2[i] = letter;
        usedSeq2.insert(std::pair<int, bool>(i, false));
    }

    //print out the first sequence, as the mutations are not being induced in this sequence
    // - it is "preserved" as it is first randomly generated
    outfile << ">Seq1" << std::endl;
    for(int i = 0; i < length; i++){
        outfile << seq1[i] << "";
    }
    outfile << "" << std::endl;

    //this loop goes through and induces the number of transitions given into sequence 2
    //as an index is chosen to induce a transition, it is marked off in the map structure
    for (int n = 0; n < transitions; ++n){
        int index = find_index(length, usedSeq2);
        seq2[index] = transition(seq2[index]);
        auto search = usedSeq2.find(index);
        search->second = true;
    }

    //this loop goes through and induces the number of transversions given into sequence 2
    //as an index is chosen to induce a transversion, it is marked off in the map structure
    for (int n = 0; n < transversions; ++n){
        int index = find_index(length, usedSeq2);
        seq2[index] = transversion(seq2[index]);
        auto search = usedSeq2.find(index);
        search->second = true;
    }

    //print out seq2 and create seq3 from the final state of seq2 after mutations
    outfile << ">Seq2" << std::endl;
    for(int i = 0; i < length; i++){
        seq3[i] = seq2[i];
        usedSeq3.insert(std::pair<int, bool>(i, false));
        outfile << seq2[i] << "";
    }
    outfile << "" << std::endl;

    //now induce mutations into sequence 3 the same way we did with sequence 2!
    for (int n = 0; n < transitions2; ++n){
        int index = find_index(length, usedSeq3);
        seq3[index] = transition(seq3[index]);
        auto search = usedSeq3.find(index);
        search->second = true;
    }

    //this loop goes through and induces the number of transversions given into sequence 3
    for (int n = 0; n < transversions2; ++n){
        int index = find_index(length, usedSeq3);
        seq3[index] = transversion(seq3[index]);
        auto search = usedSeq3.find(index);
        search->second = true;
    }

    //print out seq3
    outfile << ">Seq3" << std::endl;
    for(int i = 0; i < length; i++){
        outfile << seq3[i] << "";
    }
    outfile << "" << std::endl;


    //Do Kimura2 distance calculation!
    double** distances = new double*[numSeqs];
    for(int i = 0; i < numSeqs; ++i)
        distances[i] = new double[numSeqs];


    distances[0][0] = 0.000000;
    distances[1][1] = 0.000000;
    distances[2][2] = 0.000000;



    //compute p_f and q_f for distance between seq1 and seq2
    float p_f = (float)((float)transitions / (float)length);
    float q_f = (float)((float)transversions / (float)length);
    //fill in distance
    distances[0][1] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));
    distances[1][0] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));

    //compute p_f and q_f for distance between seq2 and seq3
    p_f = (float)((float)transitions2 / (float)length);
    q_f = (float)((float)transversions2 / (float)length);
    //compute distance
    distances[1][2] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));
    distances[2][1] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));

    //now we have to actually walk along and compare seq1 and seq3, because we don't have exact count of transitions/transversions
    for (int n = 0; n < length; ++n){
        check_index(seq1[n], seq3[n]);
    }

    //now compute the p_f, q_f, and distance for seq1 to seq3
    p_f = (float)((float)ts1_3 / (float)length);
    q_f = (float)((float)tv1_3 / (float)length);
    //compute distance
    distances[0][2] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));
    distances[2][0] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));

    return distances;
}

/*EXPECT_EQ (Formula::bla (0),  0);
EXPECT_EQ (Formula::bla (10), 20);
EXPECT_EQ (Formula::bla (50), 100);*/