#include <cstdio>
#include <cstdlib>
#include <string>
#include <vector>
#include "ArgumentHandler.hpp"
#include "ExceptionHandler.hpp"

class SequenceFileReader {
public:
    explicit SequenceFileReader(ArgumentHandler::AlphabetType alphabetType);

    ~SequenceFileReader();

    ArgumentHandler::AlphabetType getAlphabetType() const;

    std::vector<std::string> getNames() const;

    std::vector<std::string> getSequences() const;

    void readFasta(std::istream *fastaFile);

    void readStockholm(std::istream *stockholmFile);

private:
    ArgumentHandler::AlphabetType alphabetType;

    std::vector<std::string> names;

    std::vector<std::string> seqs;

    size_t numSeqs;
};
