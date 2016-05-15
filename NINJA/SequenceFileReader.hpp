/*
 * SequenceFIleReader.hpp
 *
 *  Created on: Jan 28, 2016
 *      Author: michel
 */

#include <string>
#include <stdlib.h>
#include <stdio.h>
#include <vector>
#include "ExceptionHandler.hpp"


class SequenceFileReader{
	public:
		enum AlphabetType {dna, amino, null};
		std::string** seqs;
		std::string** names;
		AlphabetType alphType;
		int seqSize;
		enum fileType {fasta, stockholm};

		SequenceFileReader (std::string *filename, AlphabetType alphTypeIn);
		~SequenceFileReader();

		fileType filetype;

		std::string **getSeqs();
		std::string **getNames();
		AlphabetType getAlphType();
};


