#include "ExceptionHandler.hpp"

#include <string>
#include <cstdlib>
#include <cstdio>
#include <vector>

class SequenceFileReader {
public:
    enum AlphabetType {
        dna, amino, null
    };
    std::string **seqs;
    std::string **names;
    AlphabetType alphType;
    int numSeqs;
    enum fileType {
        fasta, stockholm
    };

    SequenceFileReader(std::string *filename, AlphabetType alphTypeIn);

    ~SequenceFileReader();

    fileType filetype;

    std::string **getSeqs() const;

    std::string **getNames() const;

    AlphabetType getAlphType() const;
};


