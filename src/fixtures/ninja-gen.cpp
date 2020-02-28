#include <math.h>
#include <float.h>
#include <stdint.h>
#include <string>
#include <cstring>
#include <iostream>
#include <array>
#include <fstream>
#include <bitset>
#include <algorithm>
#include <map>
/*This script generates a random alignment of length 1024 nucleotides with the user's choice 
 * of number of transitions and transversions introduced into the second sequence
 * Currently does not include any gaps - they are ignored in kimura calculation anyway, so don't matter*/


int gaps = 0;
int ts1_3 = 0;  //counter to keep track of transitions between seq2 and seq3
int tv1_3 = 0;  //counter to keep track of transversions between seq2 and seq3

static const char alphabet[] =
        "A"
        "T"
        "C"
        "G";

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
    //return -1;
}

//because each nucleotide has multiple choices for transversions,
//this function randomly chooses between the two "paths" and then
//returns a different character to induce that transversion
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


//transitions are much simpler, so this just returns the correct nucleotide for
//inducing a transition in a given nucleotide
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

int main(int argc, char**argv) {
    if (argc == 5) {

    	//right now it has to be a fixed length instead of chosen by user at runtime, 
    	//otherwise the char arrays below aren't able to be initialized with that size
        const unsigned long length = 1024;
        const unsigned int transitions = atoi(argv[1]);
        const unsigned int transversions = atoi(argv[2]);
        const unsigned int transitions2 = atoi(argv[3]);
        const unsigned int transversions2 = atoi(argv[4]);

        //srand(NULL);
        char seq1[length] = {};
        char seq2[length] = {};
        char seq3[length] = {};

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
        std::cout << ">Seq1" << std::endl;
        for(int i = 0; i < length; i++){
            std::cout << seq1[i] << "";
        }
        std::cout << "\n" << std::endl;

        //this loop goes through and induces the number of transitions given into sequence 2
        //as an index is chosen to induce a transition, it is marked off in the map structure
        //std::cout << "Transitions:" << std::endl;
        for (int n = 0; n < transitions; ++n){
            int index = find_index(length, usedSeq2);
            seq2[index] = transition(seq2[index]);
            auto search = usedSeq2.find(index);
            search->second = true;
            //std::cout << search->first << std::endl;

        }

		//this loop goes through and induces the number of transversions given into sequence 2
		//as an index is chosen to induce a transversion, it is marked off in the map structure
        //std::cout << "Transversions:" << std::endl;
        for (int n = 0; n < transversions; ++n){
            int index = find_index(length, usedSeq2);
            seq2[index] = transversion(seq2[index]);
            auto search = usedSeq2.find(index);
            search->second = true;
            //std::cout << search->first << std::endl;
        }

        //print out seq2 and create seq3 from the final state of seq2 after mutations
        std::cout << ">Seq2" << std::endl;
        for(int i = 0; i < length; i++){
        	seq3[i] = seq2[i]; 
        	usedSeq3.insert(std::pair<int, bool>(i, false));
            std::cout << seq2[i] << "";
        }
        std::cout << "\n" << std::endl;


		//now induce mutations into sequence 3 the same way we did with sequence 2!
        //std::cout << "Transitions:" << std::endl;
        for (int n = 0; n < transitions2; ++n){
            int index = find_index(length, usedSeq3);
            seq3[index] = transition(seq3[index]);
            auto search = usedSeq3.find(index);
            search->second = true;
            //std::cout << search->first << std::endl;

        }

		//this loop goes through and induces the number of transversions given into sequence 3
        //std::cout << "Transversions:" << std::endl;
        for (int n = 0; n < transversions2; ++n){
            int index = find_index(length, usedSeq3);
            seq3[index] = transversion(seq3[index]);
            auto search = usedSeq3.find(index);
            search->second = true;
            //std::cout << search->first << std::endl;
        }

        //print out seq3 
        std::cout << ">Seq3" << std::endl;
        for(int i = 0; i < length; i++){
            std::cout << seq3[i] << "";
        }
        std::cout << "\n" << std::endl;

        //Do Kimura2 distance calculation!
        float dist1_2 = 0.0f;
        float dist1_3 = 0.0f;
        float dist2_3 = 0.0f;

        //compute p_f and q_f for distance between seq1 and seq2
        float p_f = (float)((float)transitions / (float)length);
        float q_f = (float)((float)transversions / (float)length);
        //compute distance
        dist1_2 = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));

        //compute p_f and q_f for distance between seq2 and seq3
        p_f = (float)((float)transitions2 / (float)length);
        q_f = (float)((float)transversions2 / (float)length);
        //compute distance
        dist2_3 = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));

        //now we have to actually walk along and compare seq1 and seq3, because we don't have exact count of transitions/transversions
        for (int n = 0; n < length; ++n){
            check_index(seq1[n], seq3[n]);
        }
        //std::cout << "counts: " << ts1_3 << ", " << tv1_3 << std::endl;

        //now compute the p_f, q_f, and distance for seq1 to seq3
        p_f = (float)((float)ts1_3 / (float)length);
        q_f = (float)((float)tv1_3 / (float)length);
        //compute distance
        dist1_3 = (float)(-0.5 * log(1.0 - 2*p_f - q_f) - 0.25 * log( 1.0-2*q_f ));

        std::cout << "Distance Matrix: \n" << "";
        std::cout << "Seq1 " << "0.000000" << " " << dist1_2 << " " << dist1_3 << std::endl;
        std::cout << "Seq2 " << dist1_2 << " " << "0.000000" << " " << dist2_3 << std::endl;
        std::cout << "Seq3 " << dist1_3 << " " << dist2_3 << " " << "0.000000" << std::endl;

        //std::cout << "Gaps:" << "";
        //std::cout << gaps << std::endl;
    }
    else
        std::cerr << "usage: ninja-gen <seq2 transitions> <seq2 transversions> <seq3 transitions> <seq3 transversions>" << std::endl;
    return 0;
}

