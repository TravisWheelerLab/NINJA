#include <iostream>
#include <string>
#include "SequenceFileReader.hpp"

SequenceFileReader::SequenceFileReader(ArgumentHandler::AlphabetType alphabetType) {
    this->alphabetType = alphabetType;
    this->numSeqs = 0;
}

SequenceFileReader::~SequenceFileReader() = default;

ArgumentHandler::AlphabetType SequenceFileReader::getAlphabetType() const {
    return this->alphabetType;
}

std::vector<std::string> SequenceFileReader::getNames() const {
    return this->names;
}

std::vector<std::string> SequenceFileReader::getSequences() const {
    return this->seqs;
}

void SequenceFileReader::readFasta(std::istream *fastaFile) {
    if (fastaFile == nullptr) {
        fastaFile = &std::cin;
    }

    std::vector<std::string> seqNames;
    std::vector<std::string> seqData;

    for (std::string line; std::getline(*fastaFile, line);) {
        if (line.empty()) {
            continue;
        }

        if (line[0] == '>') {
            // We found a name
            std::string nextName;
            auto i = 1;
            while (line.size() > i && line[i] != '\t') {
                nextName.push_back(line[i]);
                i++;
            }
            seqNames.push_back(nextName);
        } else {
            // We found a sequence
            std::string nextSeq;
            nextSeq.assign(line, 0, line.length());
            auto i = 0;
            while (line.size() > i) {
                if (line[i] == '.') {
                    // Convert "." to "-" to represent gaps
                    nextSeq.push_back('-');
                } else if (alphabetType == ArgumentHandler::dna && (line[i] == 'U' || line[i] == 'u')) {
                    // Rewrite "U" as "T" for DNA alphabet
                    nextSeq.push_back('T');
                } else {
                    // Convert to uppercase for consistency later
                    nextSeq.push_back(std::toupper(line[i]));
                }
            }
            seqData.push_back(nextSeq);
        }
    }

    numSeqs = seqNames.size();

    if (alphabetType == ArgumentHandler::unknown) {
        // Auto-detect alphabet type
        this->alphabetType = ArgumentHandler::dna;
        for (auto const &seq: seqData) {
            for (auto const &nuc: seq) {
                if (nuc != 'A' && nuc != 'C' && nuc != 'G' && nuc != 'T') {
                    this->alphabetType = ArgumentHandler::amino;
                    break;
                }
            }
        }
    }

    this->names = seqNames;
    this->seqs = seqData;
}

void SequenceFileReader::readStockholm(std::istream *stockholmFile) {
    // TODO: implement readStockholm method
}
