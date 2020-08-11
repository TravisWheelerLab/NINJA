#include "TreeBuilderManager.hpp"

#include <sys/stat.h>
#include <utility>

TreeBuilderManager::TreeBuilderManager(std::string method,
                                       std::string njTmpDir,
                                       std::string inFile,
                                       FILE *outFile,
                                       InputType inType,
                                       OutputType outType,
                                       AlphabetType alphType,
                                       CorrectionType corrType,
                                       int threads,
                                       bool useSSE) {
    this->method = std::move(method);
    this->njTmpDir = std::move(njTmpDir);
    this->inFile = std::move(inFile);
    this->outFile = outFile;
    this->inType = inType;
    this->outType = outType;
    this->alphType = alphType;
    this->corrType = corrType;
    this->names = nullptr;
    this->chars = "abcdefghijklmonpqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    this->newDistanceMethod = false;
    this->threads = threads;
    this->newDistanceMethod = useSSE;
}


std::string TreeBuilderManager::doJob() {
    int **distances = nullptr;
    float **memD = nullptr;
    float *R = nullptr;
    // all for external memory version
    int pageBlockSize = 1024; //that many ints = 4096 bytes;
    FILE *diskD = nullptr;

    int rowLength = 0;
    int firstMemCol = -1;

    int numCols = 0;

    //Runtime runtime = Runtime.getRuntime();
    long maxMemory = -1;

    TreeNode **nodes = nullptr;
    std::string treeString;

    //NinjaLogWriter.printVersion();

    int K = 0;

    /*
    #ifdef LINUX
    maxMemory = sysconf(_SC_PAGE_SIZE)*sysconf(_SC_AVPHYS_PAGES);
    #endif
    */

    SequenceFileReader *seqReader = nullptr;
    if (!this->method.compare("extmem")) {
        if (maxMemory < 1900000000) {
            fprintf(stderr, "Warning: using an external-memory variant of NINJA with less than 2GB allocated RAM.\n");
            fprintf(stderr, "The data structures of NINJA may not work well if given less than 2GB.\n");
        }
        fprintf(stderr, "Using External Memory...\n");
        njTmpDir += "treeBuilderManager";

        mkdir(njTmpDir.c_str(), 0700);
        fprintf(stderr, "created temporary directory for this run of NINJA : %s\n", njTmpDir.c_str());

        this->njTmpDir += "/";

        DistanceReaderExtMem *reader = nullptr;

        if (inType == alignment) {
            seqReader = new SequenceFileReader(&(this->inFile), (SequenceFileReader::AlphabetType) this->alphType);
            std::string **seqs = seqReader->getSeqs();
            this->names = seqReader->getNames();
            this->alphType = (TreeBuilderManager::AlphabetType) seqReader->getAlphType();
            fprintf(stderr, "Calculating distances....\n");
            auto *distCalc = new DistanceCalculator(seqs, (DistanceCalculator::AlphabetType) alphType,
                                                                  (DistanceCalculator::CorrectionType) corrType,
                                                                  seqReader->numSeqs, this->newDistanceMethod);
            K = seqReader->numSeqs;
            reader = new DistanceReaderExtMem(distCalc, K);
        } else {
            fprintf(stderr, "External memory with distances as input not allowed.\n");
            Exception::critical();
            //reader = new DistanceReaderExtMem(this->inFile);
            K = reader->K;
            this->names = new std::string *[K];
            for (int i = 0; i < K; i++)
                this->names[i] = new std::string();
        }


        R = new float[K]();
        rowLength = (K + K - 2); //that's a K*K table for the initial values, plus another K*(K-2) columns for new nodes

        long maxSize; // max amount of D stored in memory
        if (TreeBuilderExtMem::useBinPairHeaps) {
            maxSize = maxMemory / 10;
        } else {
            maxSize = maxMemory / 3;
        }

        numCols = (int) (maxSize / (4 * K));
        int numBlocks = numCols / pageBlockSize; // chops off fractional part
        if (numBlocks == 0) numBlocks = 1;  //for huge inputs, this could result in memD larger than 400MB
        numCols = numBlocks * pageBlockSize;


        if (numCols >= 2 * K - 2) {
            numCols = 2 * K - 2;
        } else {
            std::string newDir = njTmpDir + "ninja_diskD_tmp";
            FILE *tmpFile = fopen(newDir.c_str(), "w+");
            diskD = tmpFile;
        }

        memD = new float *[K];
        for (int i = 0; i < K; i++) {
            memD[i] = new float[numCols];
        }
        firstMemCol = reader->read(names, R, diskD, memD, numCols, rowLength, pageBlockSize);

        if (this->outType == dist) {
            fprintf(stderr, "Output distances with external memory not allowed.\n");
            Exception::critical();
        }

    } else {
        DistanceReader *reader = nullptr;

        if (this->inType == alignment) {
            seqReader = new SequenceFileReader(&(this->inFile), (SequenceFileReader::AlphabetType) this->alphType);
            std::string **seqs = seqReader->getSeqs();
            this->names = seqReader->getNames();
            this->alphType = (TreeBuilderManager::AlphabetType) seqReader->getAlphType();
            fprintf(stderr, "Calculating distances....\n");
            auto *distCalc = new DistanceCalculator(seqs, (DistanceCalculator::AlphabetType) alphType,
                                                                  (DistanceCalculator::CorrectionType) corrType,
                                                                  seqReader->numSeqs, newDistanceMethod);
            K = seqReader->numSeqs;
            reader = new DistanceReader(distCalc, K, this->threads);
        } else {
            reader = new DistanceReader(this->inFile);
            K = reader->K;
            this->names = new std::string *[K];
            for (int i = 0; i < K; i++)
                this->names[i] = new std::string();
        }

        distances = new int *[K];
        for (int i = 0; i < K; i++) {
            distances[i] = new int[K - i - 1];
        }


        if (this->outType == dist) {
            if (this->inType != alignment) {
                fprintf(stderr, "Input and output distances not allowed. What are you trying to do?\n");
                Exception::critical();
            }
            reader->readAndWrite(this->names, this->outFile);
            return "";
        } else {
            reader->read(this->names, distances);
        }
    }

    fprintf(stderr, "Generating tree....\n");
    int nodesSize = 0;
    TreeBuilderBinHeap *tb = nullptr;
    TreeBuilderExtMem *tb_extmem = nullptr;
    if (!this->method.compare("inmem") or !this->method.compare("default")) {
        tb = new TreeBuilderBinHeap(this->names, distances, K);
        nodes = tb->build();
        nodesSize = (tb->K * 2) - 1;

    } else if (!method.compare("extmem")) {
        tb_extmem = new TreeBuilderExtMem(names, K, R, njTmpDir, diskD, memD, numCols, firstMemCol, rowLength,
                                          maxMemory);
        nodes = tb_extmem->build();
        nodesSize = (tb_extmem->K * 2) - 1;
    }
    std::string *sb;
    if (treeString.empty()) {
        if (nodes != nullptr) {
            sb = new std::string();
            *sb = "";
            nodes[nodesSize - 1]->buildTreeString(sb);
            treeString = *sb + ";\n";
            delete sb;
        }
    }

    delete tb;
    delete tb_extmem;
    delete seqReader;
    delete[] distances;

    return (treeString);
}
