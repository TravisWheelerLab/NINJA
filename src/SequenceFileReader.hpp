/*
 * SequenceFIleReader.hpp
 *
 *  Created on: Jan 28, 2016
 *      Author: michel
 */

#include "ExceptionHandler.hpp"
#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <vector>

class SequenceFileReader {
  public:
    enum AlphabetType { dna, amino, null };
    std::string **seqs;
    std::string **names;
    AlphabetType alphType;
    int numSeqs;
    enum fileType { fasta, stockholm };

    SequenceFileReader(std::string *filename, AlphabetType alphTypeIn);
    ~SequenceFileReader();

    fileType filetype;

    std::string **getSeqs();
    std::string **getNames();
    AlphabetType getAlphType();
};
