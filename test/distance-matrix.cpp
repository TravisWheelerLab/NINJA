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

//returns an index after checking the secondary map to make sure it hasn"t already been used
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
std::basic_string<char> transversion(std::basic_string<char> nuc) {
    int num = rand() % 2;
    if(num == 0){
        if (nuc == "G")
            return "C";
        if (nuc == "C")
            return "G";
        if (nuc == "T")
            return "A";
        if (nuc == "A")
            return "T";
    }

    if(num == 1){
        if (nuc == "G")
            return "T";
        if (nuc == "C")
            return "A";
        if (nuc == "T")
            return "G";
        if (nuc == "A")
            return "C";
    }

    else{
        gaps++;
        return "-";
    }
    return 0;
}

/*Transitions are much simpler, so this just returns the correct nucleotide for
* inducing a transition in a given nucleotide*/
std::basic_string<char> transition(std::basic_string<char> nuc) {
    //A-G   C-T
    if (nuc == "G")
        return "A";
    if (nuc == "C")
        return "T";
    if (nuc == "T")
        return "C";
    if (nuc == "A")
        return "G";
    return 0;
}

/*this method checks the given index between seq1 and seq3 because they are the only pair of sequences
which the number of transitions/transversions is not known between*/
void check_index(std::basic_string<char> nuc1, std::basic_string<char> nuc2){
    if(nuc1 == nuc2){
        return;
    }
    else if(nuc1 == "G"){
        if(nuc2 == "A")
            ts1_3++;
        else if(nuc2 == "C" || nuc2 == "T")
            tv1_3++;
        return;
    }

    else if (nuc1 == "C") {
        if (nuc2 == "T")
            ts1_3++;
        else if (nuc2 == "G" || nuc2 == "A")
            tv1_3++;
        return;
    }
    else if (nuc1 == "T"){
        if(nuc2 == "C")
            ts1_3++;
        else if(nuc2 == "A" || nuc2 == "G")
            tv1_3++;
        return;
    }

    else if (nuc1 == "A"){
        if(nuc2 == "G")
            ts1_3++;
        else if(nuc2 == "T" || nuc2 == "C")
            tv1_3++;
        return;
    }
}
//This function does the kimura2 distance calc - removed from generation function to do its own thing here
double** kimura_calc(double** distances, const unsigned int transitions, const unsigned int transversions, const unsigned int transitions2,
                     const unsigned int transversions2, const unsigned long length){
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

    //now compute the p_f, q_f, and distance for seq1 to seq3
    p_f = (float)((float)ts1_3 / (float)length);
    q_f = (float)((float)tv1_3 / (float)length);

    //compute distance
    distances[0][2] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));
    distances[2][0] = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));

    return distances;
}

void generate_all(std::string* seq1, std::string* seq2, std::string* seq3, const unsigned long length, int allowed_mutations, double** distances/*, std::ofstream outfile*/){
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

    //create seq3 from the final state of seq2 after mutations
    for(int i = 0; i < length; i++){
        seq3[i] = seq2[i];
        usedSeq3.insert(std::pair<int, bool>(i, false));
    }

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

    //now we have to actually walk along and compare seq1 and seq3, because we don"t have exact count of transitions/transversions
    for (int n = 0; n < length; ++n){
        check_index(seq1[n], seq3[n]);
    }
    return;
}

TEST(DistCalcTest, DistMatrix10){
    DistanceCalculator::AlphabetType alphType = DistanceCalculator::dna;
    DistanceCalculator::CorrectionType corrType = DistanceCalculator::Kimura2;

/*  //This was the previous workflow for creating a distance calculator, pulled from treebuildermanager
 * - not 100% sure it's completely unneeded yet and it was hard to track down the first time...
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
    */

    //now do random generation for comparison!
    const unsigned long length = 10;
    //char seq1[length] = {};
    std::string* seq1 = new std::string[length];
    //char seq2[length] = {};
    std::string* seq2 = new std::string[length];
    //char seq3[length] = {};
    std::string* seq3 = new std::string[length];
    int allowed_mutations = 2;
    //std::ofstream outfile = new std::ofstream("../src/fixtures/10-test.fa",std::ios::out);
    double** distances = new double*[numSeqs];
    for(int i = 0; i < numSeqs; ++i)
        distances[i] = new double[numSeqs];
    distances[0][0] = 0.000000;
    distances[1][1] = 0.000000;
    distances[2][2] = 0.000000;

    generate_all(seq1, seq2, seq3, length, allowed_mutations, distances);

    std::string** seqNames = new std::string*[numSeqs];
    std::string** sequences = new std::string*[numSeqs];
    seqNames[0] = new std::string();
    seqNames[0]->assign("Seq1");
    seqNames[1] = new std::string();
    seqNames[1]->assign("Seq2");
    seqNames[2] = new std::string();
    seqNames[2]->assign("Seq3");

    sequences[0] = seq1;
    sequences[1] = seq2;
    sequences[2] = seq3;

    bool newCalculation = true;
    DistanceCalculator* distCalc = new DistanceCalculator(sequences, alphType, corrType, numSeqs, newCalculation);
}

/*EXPECT_EQ (Formula::bla (0),  0);
EXPECT_EQ (Formula::bla (10), 20);
EXPECT_EQ (Formula::bla (50), 100);*/